enum State {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    redis_connection: redis::aio::MultiplexedConnection,
}

const CIRCUIT_BREAKER_KEY: &'static str = "circuit_breaker_state";

impl CircuitBreaker {
    pub async fn init(mut redis_connection: redis::aio::MultiplexedConnection) -> Self {
        let exists = redis::cmd("EXISTS")
            .query_async::<i32>(&mut redis_connection)
            .await
            .unwrap_or(0) > 0;

        let mut circuit_breaker = CircuitBreaker { redis_connection };

        if !exists {
            circuit_breaker.save_state_to_redis(State::Closed).await;
        }
        circuit_breaker
    }

    async fn get_state_from_redis(&mut self) -> State {
        // let state: String = self.redis_connection.get(CIRCUIT_BREAKER_KEY).unwrap_or_else(|_| "CLOSED".to_string());
        let state = redis::cmd("GET")
            .arg(CIRCUIT_BREAKER_KEY)
            .query_async::<String>(&mut self.redis_connection)
            .await
            .unwrap_or_else(|_| "CLOSED".to_string());

        match state.as_str() {
            "OPEN" => State::Open,
            "HALF_OPEN" => State::HalfOpen,
            _ => State::Closed,
        }
    }

    async fn save_state_to_redis(&mut self, state: State) {
        let value = match state {
            State::Open => "OPEN",
            State::HalfOpen => "HALF_OPEN",
            State::Closed => "CLOSED",
        };
        // let _: () = self.redis_connection.set(CIRCUIT_BREAKER_KEY, value).unwrap();
        
        let _: () = redis::cmd("SET")
            .arg(CIRCUIT_BREAKER_KEY)
            .arg(value)
            .query_async::<()>(&mut self.redis_connection)
            .await
            .unwrap();
    }

    pub async fn open(&mut self) {
        self.save_state_to_redis(State::Open).await;
    }

    pub async fn close(&mut self) {
        self.save_state_to_redis(State::Closed).await;
    }

    async fn half_open(&mut self) {
        self.save_state_to_redis(State::HalfOpen).await;
    }
}