# Multi-stage Dockerfile for repo_roller_api
# Optimized for small image size and security

# Stage 1: Build
FROM rust:1.90-bookworm@sha256:3914072ca0c3b8aad871db9169a651ccfce30cf58303e5d6f2db16d1d8a7e58f AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY deny.toml ./

# Copy all crates (needed for workspace build)
COPY crates/ crates/

# Build the API binary in release mode
RUN cargo build --release --package repo_roller_api

# Stage 2: Runtime
FROM debian:bookworm-slim@sha256:e899040a73d36e2b36fa33216943539d9957cba8172b858097c2cabcdb20a3e2

# Install runtime dependencies (OpenSSL for HTTPS, git for template cloning)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 reporoller

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/repo_roller_api /app/repo_roller_api

# Change ownership to non-root user
RUN chown -R reporoller:reporoller /app

# Switch to non-root user
USER reporoller

# Expose API port
EXPOSE 8080

# Set environment variables with defaults
ENV RUST_LOG=info
ENV API_HOST=0.0.0.0
ENV API_PORT=8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/repo_roller_api", "--health-check"]

# Run the API server
CMD ["/app/repo_roller_api"]
