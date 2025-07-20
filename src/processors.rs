use reqwest::Response;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::circuit_breaker;
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

#[async_trait::async_trait]
trait Processor {
    async fn send_payment(&self, body: SendPaymentRequestBody) -> SendPaymentResponseBody {
        let client = reqwest::Client::new();
        let url = format!("{}/payments", self.get_processor_url());

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .expect("Failed to send payment");

        response.json().await.expect("Failed to parse response")
    }

    async fn health_check(&self) -> Result<HealthCheckResponseBody, reqwest::Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/payments/service-health", self.get_processor_url());

        let response = client.get(&url).send().await?;
        let body = response.json().await?;
        Ok(body)
    }

    async fn payments_details(&self, id: String) -> PaymentsDetailsResponseBody {
        let client = reqwest::Client::new();
        let url = format!("{}/payments/{}", self.get_processor_url(), id);

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

    fn get_processor_url(&self) -> String;
}

struct DefaultProcessor;

#[async_trait::async_trait]
impl Processor for DefaultProcessor {
    fn get_processor_url(&self) -> String {
        std::env::var("PAYMENT_PROCESSOR_URL_DEFAULT")
            .unwrap_or_else(|_| "http://payment-processor-default:8080".to_string())
    }
}

struct FallbackProcessor;

#[async_trait::async_trait]
impl Processor for FallbackProcessor {
    fn get_processor_url(&self) -> String {
        std::env::var("PAYMENT_PROCESSOR_URL_FALLBACK")
            .unwrap_or_else(|_| "http://payment-processor-fallback:8080".to_string())
    }
}

fn load_processor(processor_name: &str) -> Option<Box<dyn Processor + Send + Sync>> {
    match processor_name {
        "default" => Some(Box::new(DefaultProcessor)),
        "fallback" => Some(Box::new(FallbackProcessor)),
        _ => None,
    }
}

pub async fn health_check_worker(processor_name: String, mut redis_connection: redis::aio::MultiplexedConnection) {
    loop {
        let processor = if let Some(processor) = load_processor(&processor_name) {
            processor
        } else {
            tracing::error!("Unknown processor: {}", processor_name);
            return;
        };

        tracing::info!("Health check worker for {}", processor_name);

        let mut cb = circuit_breaker::load_state_from_redis(&processor_name, &mut redis_connection).await;

        if cb.is_request_allowed() {
            let res = processor.health_check().await;

            if res.is_ok() {
                let health_check_response = res.unwrap();
                cb.on_request_result(!health_check_response.failing);
            } else {
                cb.on_request_result(false);
            }
        }

        circuit_breaker::save_state_to_redis(cb, &mut redis_connection).await;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

pub async fn send_queue_payments_worker(mut redis_connection: redis::aio::MultiplexedConnection) {
    loop {
        tracing::info!("Send queue payments worker");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}