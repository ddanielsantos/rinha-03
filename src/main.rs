#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    let state = AppState {};

    let router = payments::get_router().with_state(state);
    let address = "0.0.0.0";

    let port = "3000";
    let address = format!("{}:{}", address, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}

mod payments {
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
        total_amout: f64,
    }

    struct PaymentsSummaryResponseBody {
        default: ServiceInfo,
        fallback: ServiceInfo,
    }

    async fn payments_summary_handler(State(state): State<AppState>) -> impl IntoResponse {
        "summary"
    }

    pub fn get_router() -> Router<AppState> {
        Router::new()
            .route("/payments", post(payments_handler))
            .route("/payments-summary", get(payments_summary_handler))
    }

    mod processors {
        struct HealthCheckResponseBody {
            failing: bool,
            min_response_time: u64,
        }

        struct PaymentsDetailsResponseBody {
            correlation_id: String,
            amount: f64,
            requested_at: String, // TODO: make it ISO UTC, like 2025-07-15T12:34:56.000Z
        }

        trait Processor {
            fn send_payment();
            fn health_check() -> HealthCheckResponseBody;
            fn payments_details(id: String) -> PaymentsDetailsResponseBody;
        }
    }
}
