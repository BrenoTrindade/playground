# Stage 1: Build & Dependency Caching
FROM rust:1.99-bookworm AS builder

WORKDIR /app

# Create dummy main to pre-cache dependencies
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy real source code and build final binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Strip debug symbols to reduce binary size
RUN strip target/release/api-rust

# Stage 2: Minimal, Non-Root Runtime
FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Run as non-root user for security
RUN useradd -u 10001 -m -s /bin/bash appuser

WORKDIR /home/appuser
COPY --from=builder /app/target/release/api-rust /usr/local/bin/api-rust

USER appuser

EXPOSE 3000

ENV PORT=3000
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/api-rust"]
