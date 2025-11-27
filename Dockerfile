# Multi-stage Dockerfile for Nexus Functions
# Stage 1: Build the Rust application
FROM rust:1.83-bookworm AS builder

WORKDIR /build

# Install WASM target for building WASM modules
RUN rustup target add wasm32-wasi wasm32-unknown-unknown

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY runtime/Cargo.toml ./runtime/
COPY nats_integration/Cargo.toml ./nats_integration/
COPY observability/Cargo.toml ./observability/
COPY cli/Cargo.toml ./cli/
COPY config/Cargo.toml ./config/

# Create dummy source files to cache dependencies
RUN mkdir -p core/src runtime/src nats_integration/src observability/src cli/src config/src && \
    echo "fn main() {}" > core/src/lib.rs && \
    echo "fn main() {}" > runtime/src/lib.rs && \
    echo "fn main() {}" > nats_integration/src/lib.rs && \
    echo "fn main() {}" > observability/src/lib.rs && \
    echo "fn main() {}" > cli/src/main.rs && \
    echo "fn main() {}" > config/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release --bin nexus

# Remove dummy sources
RUN rm -rf core/src runtime/src nats_integration/src observability/src cli/src config/src

# Copy actual source code
COPY core ./core
COPY runtime ./runtime
COPY nats_integration ./nats_integration
COPY observability ./observability
COPY cli ./cli
COPY config ./config

# Build the actual application
RUN cargo build --release --bin nexus

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 nexus && \
    mkdir -p /app /app/functions /app/nats-data && \
    chown -R nexus:nexus /app

WORKDIR /app

# Copy the compiled binary from builder
COPY --from=builder /build/target/release/nexus /usr/local/bin/nexus

# Copy default configuration (optional - can be overridden)
COPY --chown=nexus:nexus examples/nexus.yaml.example /app/nexus.yaml.example

# Switch to non-root user
USER nexus

# Expose ports
# 8080: Nexus Functions HTTP API
# 4222: NATS client connections (if embedded NATS is used)
# 8222: NATS monitoring (if embedded NATS is used)
EXPOSE 8080 4222 8222

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info \
    NEXUS_HOST=0.0.0.0 \
    NEXUS_PORT=8080

# Default command
CMD ["nexus", "dev"]
