# ----- builder stage -----
FROM rust:1.95-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build real sources
COPY src ./src
RUN touch src/main.rs && cargo build --release && \
    strip target/release/fishing-map-backend

# ----- runtime stage -----
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fishing-map-backend /usr/local/bin/server

ENV PORT=8765
EXPOSE 8765
CMD ["/usr/local/bin/server"]
