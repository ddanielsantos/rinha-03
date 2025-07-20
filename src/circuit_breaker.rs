use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize)]
enum State {
    Closed,
    Open,
    HalfOpen,
}


#[derive(Serialize, Deserialize)]
pub struct CircuitBreaker {
    name: String,
    state: State,
    opened_at: Option<OffsetDateTime>,
}

impl CircuitBreaker {
    pub fn new(name: String) -> CircuitBreaker {
        CircuitBreaker {
            name,
            state: State::Closed,
            opened_at: None,
        }
    }
}

impl CircuitBreaker {
    pub fn trip(&mut self) {
        tracing::info!("Circuit breaker tripped");
        self.state = State::Open;
        self.opened_at = Some(OffsetDateTime::now_utc());
    }

    pub fn reset(&mut self) {
        tracing::info!("Circuit breaker reset");
        self.state = State::Closed;
        self.opened_at = None;
    }

    pub fn on_request_result(&mut self, success: bool) {
        match self.state {
            State::Open => {
                // If the circuit breaker is open, we do not allow requests to proceed.
            }
            State::HalfOpen => {
                if success {
                    self.reset();
                } else {
                    self.trip();
                }
            }
            State::Closed => {
                if !success {
                    self.trip();
                }
            }
        }
    }

    pub fn is_request_allowed(&self) -> bool {
        match self.state {
            State::Open => false,
            State::HalfOpen | State::Closed => true,
        }
    }
}

pub async fn load_state_from_redis(processor_name: &String, redis_con: &mut redis::aio::MultiplexedConnection) -> CircuitBreaker {
    let key = format!("circuit_breaker:{}", processor_name);
    match redis::cmd("GET")
        .arg(&key)
        .query_async::<String>(redis_con)
        .await
    {
        Ok(value) => {
            serde_json::from_str(&value).unwrap_or_else(|_| CircuitBreaker::new(processor_name.clone()))
        }
        Err(_) => CircuitBreaker::new(processor_name.clone()),
    }
}

pub async fn save_state_to_redis(cb: CircuitBreaker, redis_con: &mut redis::aio::MultiplexedConnection) {
    let key = format!("circuit_breaker:{}", cb.name);
    let value = serde_json::to_string(&cb).expect("Failed to serialize CircuitBreaker");

    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .query_async::<()>(redis_con)
        .await
        .expect("Failed to save CircuitBreaker state to Redis");
}

mod tests {
    use crate::circuit_breaker::CircuitBreaker;

    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new("test_processor".to_string());

        assert!(cb.is_request_allowed());
        cb.trip();
        assert!(!cb.is_request_allowed());

        cb.on_request_result(true);
        // assert!(cb.is_request_allowed());
    }
}