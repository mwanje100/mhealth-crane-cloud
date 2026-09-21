FROM rust:1.80-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/edge_node /app/edge_node
EXPOSE 3030
CMD ["/app/edge_node"]
