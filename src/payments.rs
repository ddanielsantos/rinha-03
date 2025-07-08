use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::AppState;

struct PaymentsRequestBody {
    correlation_id: String,
    amount: f64,
}

async fn payments_handler(State(state): State<AppState>) -> impl IntoResponse {
    "paid"
}

struct ServiceInfo {
    total_requests: u64,
    total_amount: f64,
}

struct PaymentsSummaryResponseBody {
    default: ServiceInfo,
    fallback: ServiceInfo,
}

async fn payments_summary_handler(State(state): State<AppState>) -> impl IntoResponse {
    "summary"
}

async fn internal_check_handler(State(state): State<AppState>) -> impl IntoResponse {
    format!("server_id: {}", std::env::var("SERVER_ID").unwrap_or_else(|_| "default_server".to_string()))
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/payments", post(payments_handler))
        .route("/payments-summary", get(payments_summary_handler))
        .route("/internal/check", get(internal_check_handler))
}

pub async fn send_queue_payments_worker() {
    loop {
        println!("Sending queue payments");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
