use bb8_redis::RedisConnectionManager;
use tokio::task;

mod payments;
mod processors;
mod circuit_breaker;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    let client = redis::Client::open("redis://rinha_redis")
        .expect("Failed to create Redis client");
    let con = client.get_multiplexed_async_connection().await
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
