# Multi-stage Dockerfile for building and running the myblog Rust web app

### Builder stage
FROM rust:latest AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev libpq-dev ca-certificates git curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest and lock first to leverage docker layer cache for dependencies
COPY Cargo.toml Cargo.lock ./
COPY sqlx-data.json ./

# Create a dummy src to allow cargo to fetch dependencies before copying whole project
RUN mkdir -p src && printf "fn main() {}\n" > src/main.rs
RUN cargo fetch --locked

# Copy rest of the source
COPY . .

# Build in release mode
RUN cargo build --release


### Runtime stage
FROM debian:latest

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and assets from builder
COPY --from=builder /app/target/release/blog /usr/local/bin/blog
COPY --from=builder /app/static /app/static
COPY --from=builder /app/views /app/views
COPY --from=builder /app/migrations /app/migrations

ENV RUST_LOG=info
EXPOSE 8080

# The application expects DATABASE_URL environment variable to be set.
CMD ["/usr/local/bin/blog"]
