ARG RUST_VERSION=1.88

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --bin rinha-03

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/rinha-03 /bin/server
EXPOSE 3000

ENTRYPOINT ["/bin/server"]