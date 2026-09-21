# Stage 1: Build the Rust Binary
FROM rust:1.80-slim as builder
WORKDIR /usr/src/app

# Install standard C compilation tooling
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential && rm -rf /var/lib/apt/lists/*

COPY . .

# Compile release binary named 'mhealth' (matching Cargo.toml)
RUN cargo build --release --bin mhealth

# Stage 2: Lightweight Runtime Environment
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy compiled binary from builder stage
COPY --from=builder /usr/src/app/target/release/mhealth /app/mhealth

ENV PORT=3030
EXPOSE 3030

CMD ["/app/mhealth"]
