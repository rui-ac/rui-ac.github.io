# Build stage: compile Rust binary
FROM rust:slim AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    ca-certificates \
    && pip3 install --break-system-packages -U gallery-dl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/backend ./backend

CMD ["./backend"]
