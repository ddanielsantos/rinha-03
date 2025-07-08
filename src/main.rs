use tokio::task;

mod payments;
mod processors;


#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    task::spawn(payments::send_queue_payments_worker());
    task::spawn(processors::health_check_worker());

    let state = AppState {};

    let router = payments::get_router().with_state(state);
    let address = "0.0.0.0";

    let port = "3000";
    let address = format!("{}:{}", address, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
