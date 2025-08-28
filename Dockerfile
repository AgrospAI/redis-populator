## Stage 1 · Building
FROM rust:1.88-slim AS builder

ARG WORKDIR=/app
WORKDIR $WORKDIR

RUN apt update && \
    apt install -y \
        libssl-dev pkg-config && \
    rustup target add x86_64-unknown-linux-gnu && \
    update-ca-certificates

# Copy sources
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "populator"

RUN cargo build --release

## Stage 2 · Running
FROM debian:bookworm-slim

ARG WORKDIR=/app
WORKDIR $WORKDIR

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

USER populator:populator

# Copy user data from builder
COPY --from=builder /etc/passwd /etc/group /etc/

# Copy binary
COPY --from=builder --chown=populator:populator \
    $WORKDIR/target/release/redis-populator \
    $WORKDIR/redis-populator

ENTRYPOINT [ "/app/redis-populator" ]