# Multi-stage Dockerfile for Diffusion Server

# Stage 1: Build
FROM rust:1.75 as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Copy source
COPY proto ./proto
COPY src ./src
COPY config ./config

# Build for release
RUN cargo build --release

# Stage 2: Runtime
FROM nvidia/cuda:12.1.0-runtime-ubuntu22.04

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/diffusion-server /app/diffusion-server

# Copy config
COPY config /app/config

# Create necessary directories
RUN mkdir -p /app/models /app/cache /app/output

# Expose ports
EXPOSE 50051 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD grpc_health_probe -addr=localhost:50051 || exit 1

# Run the server
CMD ["/app/diffusion-server"]
