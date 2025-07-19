use axum::{extract::State, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaymentsRequestBody {
    correlation_id: String,
    amount: f64,
}

async fn payments_handler(State(mut state): State<AppState>, Json(body): Json<PaymentsRequestBody>) -> impl IntoResponse {
    let _ = redis::cmd("LPUSH")
        .arg("payments_queue")
        .arg(serde_json::to_string(&body).unwrap())
        .query_async::<()>(&mut state.redis_connection)
        .await
        .expect("Failed to push payment to Redis queue");

    ()
}

struct ServiceInfo {
    total_requests: u64,
    total_amount: f64,
}

struct PaymentsSummaryResponseBody {
    default: ServiceInfo,
    fallback: ServiceInfo,
}

async fn payments_summary_handler(State(_state): State<AppState>) -> impl IntoResponse {
    "summary"
}

async fn internal_check_handler(State(_state): State<AppState>) -> impl IntoResponse {
    format!(
        "server_id: {}",
        std::env::var("SERVER_ID").unwrap_or_else(|_| "default_server".to_string())
    )
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/payments", post(payments_handler))
        .route("/payments-summary", get(payments_summary_handler))
        .route("/internal/check", get(internal_check_handler))
}
