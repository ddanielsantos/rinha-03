use crate::circuit_breaker::CircuitBreaker;
use reqwest::Response;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SendPaymentRequestBody {
    correlation_id: String,
    amount: f64,
    requested_at: String, // TODO: make it ISO UTC, like 2025-07-15T12:34:56.000Z
}

#[derive(Deserialize)]
struct SendPaymentResponseBody {
    message: String,
}

#[derive(Deserialize)]
struct HealthCheckResponseBody {
    failing: bool,
    min_response_time: u64,
}

#[derive(Deserialize)]
struct PaymentsDetailsResponseBody {
    correlation_id: String,
    amount: f64,
    requested_at: String, // TODO: make it ISO UTC, like 2025-07-15T12:34:56.000Z
}

trait Processor {
    async fn send_payment(&self, body: SendPaymentRequestBody) -> SendPaymentResponseBody {
        let client = reqwest::Client::new();
        let url = format!("{}/payments", Self::get_processor_url());

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .expect("Failed to send payment");

        response.json().await.expect("Failed to parse response")
    }

    async fn health_check(&self) -> HealthCheckResponseBody {
        let client = reqwest::Client::new();
        let url = format!("{}/payments/service-health", Self::get_processor_url());

        let response: Response = client
            .get(&url)
            .send()
            .await
            .expect("Failed to perform health check");

        response
            .json()
            .await
            .expect("Failed to parse health check response")
    }

    async fn payments_details(&self, id: String) -> PaymentsDetailsResponseBody {
        let client = reqwest::Client::new();
        let url = format!("{}/payments/{}", Self::get_processor_url(), id);

        let response: Response = client
            .get(&url)
            .send()
            .await
            .expect("Failed to get payment details");

        response
            .json()
            .await
            .expect("Failed to parse payment details response")
    }

    fn get_processor_url() -> String;
}

struct DefaultProcessor;

impl Processor for DefaultProcessor {
    fn get_processor_url() -> String {
        "http://default-processor:3000".to_string()
    }
}

pub async fn health_check_worker(redis_connection: redis::aio::MultiplexedConnection) {
    let processor = DefaultProcessor;
    let mut circuit_breaker = CircuitBreaker::init(redis_connection).await;

    loop {
        match processor.health_check().await {
            HealthCheckResponseBody { failing: false, .. } => {
                circuit_breaker.close().await;
            }
            HealthCheckResponseBody { failing: true, .. } => {
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
