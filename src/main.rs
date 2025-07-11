use std::env;
use tokio::task;

mod circuit_breaker;
mod payments;
mod processors;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
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

    task::spawn(payments::send_queue_payments_worker());
    task::spawn(processors::health_check_worker(con));

    let state = AppState {};

    let router = payments::get_router().with_state(state);
    let address = "0.0.0.0";

    let port = "3000";
    let address = format!("{}:{}", address, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
