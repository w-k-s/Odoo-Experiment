# ---- Build stage ----
FROM rust:1-bookworm AS builder

# Mirror the local layout so embed_migrations!("../../../../migrations") resolves
# relative to the project root, same as it does on the host.
WORKDIR /project
COPY migrations ./migrations
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY backend/crates ./backend/crates

WORKDIR /project/backend
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /project/backend/target/release/server /usr/local/bin/loyalty-backend

EXPOSE 8000
CMD ["loyalty-backend"]
