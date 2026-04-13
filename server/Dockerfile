FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests for dependency caching
COPY server/Cargo.toml server/Cargo.lock* ./

# Create dummy src matching lib+bin layout so cargo resolves deps
# Also create the sibling migrations dir that sqlx::migrate!("../migrations") expects
RUN mkdir -p src /migrations && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs

# Cache deps (ignore errors from missing full source)
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy real source — migrations go to /migrations so that
# sqlx::migrate!("../migrations") (relative to crate root /app) resolves to /migrations
COPY server/src ./src
COPY migrations /migrations

# Rebuild with real source
RUN touch src/lib.rs src/main.rs && cargo build --release

# Runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kanva-server /usr/local/bin/
# Migrations are embedded in the binary by sqlx::migrate! at compile time — no files needed at runtime

EXPOSE 8080
CMD ["kanva-server"]
