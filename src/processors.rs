use crate::circuit_breaker::CircuitBreaker;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::payments::PaymentsRequestBody;

#[derive(Serialize)]
struct SendPaymentRequestBody {
    #[serde(rename = "correlationId")]
    correlation_id: String,
    amount: f64,
    #[serde(rename = "requestedAt")]
    #[serde(with = "time::serde::rfc3339")]
    requested_at: OffsetDateTime,
}

impl SendPaymentRequestBody {
    fn from_payments_request_body(body: PaymentsRequestBody) -> Self {
        Self {
            correlation_id: body.correlation_id,
            amount: body.amount,
            requested_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Deserialize)]
struct SendPaymentResponseBody {
    message: String,
}

#[derive(Deserialize)]
#[derive(Debug)]
struct HealthCheckResponseBody {
    failing: bool,
    #[serde(rename = "minResponseTime")]
    min_response_time: u64,
}

#[derive(Deserialize)]
struct PaymentsDetailsResponseBody {
    #[serde(rename = "correlationId")]
    correlation_id: String,
    amount: f64,
    #[serde(rename = "requestedAt")]
    requested_at: String,
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
        std::env::var("PAYMENT_PROCESSOR_URL_DEFAULT")
            .unwrap_or_else(|_| "http://payment-processor-default:8080".to_string())
    }
}

pub async fn health_check_worker(redis_connection: redis::aio::MultiplexedConnection) {
    let processor = DefaultProcessor;
    let mut circuit_breaker = CircuitBreaker::init(redis_connection).await;

    loop {
        let body = processor.health_check().await;

        if !body.failing {
            circuit_breaker.close().await;
            tracing::info!("Circuit breaker closed, processor is healthy");
        } else {
            circuit_breaker.open().await;
            tracing::warn!("Circuit breaker opened, processor is unhealthy");
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

pub async fn send_queue_payments_worker(mut redis_connection: redis::aio::MultiplexedConnection) {
    loop {
        let payment = redis::cmd("LPOP")
            .arg("payments_queue")
            .query_async::<String>(&mut redis_connection)
            .await;
        match payment {
            Ok(value) => {
                if let Ok(body) = serde_json::from_str::<PaymentsRequestBody>(value.as_str()) {
                    let send_body = SendPaymentRequestBody::from_payments_request_body(body);
                    let processor = DefaultProcessor;
                    let response = processor.send_payment(send_body).await;
                    tracing::info!("Payment sent: {}", response.message);
                } else if let Err(err) = serde_json::from_str::<PaymentsRequestBody>(value.as_str()) {
                    tracing::error!("Failed to parse payment from queue: {}", err);
                }
            }
            Err(err) => {
                if err.kind() == redis::ErrorKind::TypeError {
                    tracing::info!("No payments in queue, waiting for new payments");
                } else {
                    tracing::error!("Failed to pop payment from queue: {}", err);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}