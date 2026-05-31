FROM rust:1.77-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY runtime ./runtime
COPY server ./server
RUN cargo build --release --locked -p arcflow-relay

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/arcflow-relay /usr/local/bin/arcflow-relay
EXPOSE 8090
ENV ARCFLOW_RELAY_PORT=8090
HEALTHCHECK CMD curl -f http://localhost:8090/health || exit 1
ENTRYPOINT ["arcflow-relay"]
