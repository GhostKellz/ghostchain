# GhostChain Workspace Multi-Service Dockerfile
# Multi-stage build for ghostd and walletd with optimal image size

# Build stage
FROM rust:1.75-bullseye as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    cmake \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY shared/Cargo.toml ./shared/
COPY ghostd/Cargo.toml ./ghostd/
COPY walletd/Cargo.toml ./walletd/
COPY integration-tests/Cargo.toml ./integration-tests/

# Create dummy src directories to build dependencies
RUN mkdir -p core/src shared/src ghostd/src walletd/src integration-tests/src && \
    echo "fn main() {}" > ghostd/src/main.rs && \
    echo "fn main() {}" > walletd/src/main.rs && \
    echo "pub fn dummy() {}" > core/src/lib.rs && \
    echo "pub fn dummy() {}" > shared/src/lib.rs && \
    echo "pub fn dummy() {}" > integration-tests/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf */src

# Copy actual source code
COPY core/src ./core/src
COPY shared/src ./shared/src
COPY ghostd/src ./ghostd/src
COPY walletd/src ./walletd/src
COPY integration-tests/ ./integration-tests/

# Build the workspace
RUN cargo build --release --workspace

# Runtime stage for ghostd
FROM debian:bullseye-slim as ghostd

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create ghostd user
RUN useradd -m -s /bin/bash ghostd

# Create necessary directories
RUN mkdir -p /opt/ghostchain/{data,logs,config} && \
    chown -R ghostd:ghostd /opt/ghostchain

# Copy the binary from builder stage
COPY --from=builder /app/target/release/ghostd /usr/local/bin/ghostd

# Switch to ghostd user
USER ghostd
WORKDIR /opt/ghostchain

# Expose ports (RPC, API, P2P, QUIC)
EXPOSE 8545 8547 8546 8549

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8547/api/v1/health || exit 1

# Default command
CMD ["ghostd", "start", "--bind-address", "0.0.0.0:8545"]

# Runtime stage for walletd
FROM debian:bullseye-slim as walletd

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create walletd user
RUN useradd -m -s /bin/bash walletd

# Create necessary directories
RUN mkdir -p /opt/ghostchain/{wallets,identities,keys,config,logs} && \
    chown -R walletd:walletd /opt/ghostchain

# Copy the binary from builder stage
COPY --from=builder /app/target/release/walletd /usr/local/bin/walletd

# Switch to walletd user
USER walletd
WORKDIR /opt/ghostchain

# Expose ports (API, QUIC)
EXPOSE 8548 8550

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8548/health || exit 1

# Default command
CMD ["walletd", "start", "--bind-address", "0.0.0.0:8548"]