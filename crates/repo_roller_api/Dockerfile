# Multi-stage Dockerfile for repo_roller_api
# Optimized for small image size and security

# Stage 1: Build
FROM rust:1.100-bookworm@sha256:198cd28b4ba92b20fe849d5d8b119ace1bd28423d9e82e9c4de5ee1300d453a0 AS builder

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
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc

# Install runtime dependencies (OpenSSL for HTTPS, git for template cloning,
# wget for the Docker health check probe against GET /api/v1/health)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    git \
    wget \
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
    CMD ["wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8080/api/v1/health"]

# Run the API server
CMD ["/app/repo_roller_api"]
