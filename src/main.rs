use std::env;
use tokio::task;

mod circuit_breaker;
mod payments;
mod processors;

#[derive(Clone)]
struct AppState {
    redis_connection: redis::aio::MultiplexedConnection,
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();

    #[cfg(not(debug_assertions))]
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://rinha_redis".to_string());
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let con = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to create Redis connection");

    {
        let _: () = redis::cmd("PING")
            .query_async::<()>(&mut con.clone())
            .await
            .expect("Failed to ping Redis");
        tracing::info!("Redis connection established");
    }

    let queue_con = con.clone();
    task::spawn(async move { processors::send_queue_payments_worker(queue_con).await });

    let default_health_con = con.clone();
    task::spawn(async move { processors::health_check_worker("default".to_string(), default_health_con).await });

    let fallback_health_con = con.clone();
    task::spawn(async move { processors::health_check_worker("fallback".to_string(), fallback_health_con).await });

    let state = AppState {
        redis_connection: con,
    };

    let router = payments::get_router().with_state(state);
    let address = "0.0.0.0";

    let port = "3000";
    let address = format!("{}:{}", address, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
