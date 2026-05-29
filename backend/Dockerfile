# ---- Build stage ----
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies: copy manifests first, build a dummy target, then the source.
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Now copy the real sources + embedded migrations and build for real.
COPY migrations ./migrations
COPY src ./src
# Touch main so cargo rebuilds with the real source.
RUN touch src/main.rs && cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

# diesel's postgres backend links libpq dynamically at runtime.
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/loyalty-backend /usr/local/bin/loyalty-backend

EXPOSE 8000
CMD ["loyalty-backend"]
