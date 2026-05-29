FROM rust:1.77-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY runtime ./runtime
COPY server ./server
RUN cargo build --release --locked -p arcflow-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN groupadd --gid 1000 arcflow && useradd --uid 1000 --gid arcflow --create-home arcflow
COPY --from=builder /build/target/release/arcflow-server /usr/local/bin/arcflow-server
USER arcflow
EXPOSE 8080
ENV ARCFLOW_PORT=8080
HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1
ENTRYPOINT ["arcflow-server"]
