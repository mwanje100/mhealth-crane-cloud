# Multi-stage Docker build for minimal deployment container size
FROM rust:1.80-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin edge_node

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/edge_node /app/edge_node

ENV PORT=3030
EXPOSE 3030

CMD ["/app/edge_node"]
