# Multi-stage build for production GhostChain
FROM zigtools/zig:0.15.0-dev as builder

WORKDIR /app

# Copy build files
COPY build.zig build.zig.zon ./
COPY src/ ./src/
COPY proto/ ./proto/

# Build for production with optimizations
RUN zig build -Doptimize=ReleaseFast -Dtarget=x86_64-linux-gnu

# Production image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    iputils-ping \
    net-tools \
    && rm -rf /var/lib/apt/lists/*

# Create ghostchain user
RUN useradd -r -s /bin/false -m -d /var/lib/ghostchain ghostchain

# Copy binaries from builder
COPY --from=builder /app/zig-out/bin/ghostchain /usr/local/bin/
RUN chmod +x /usr/local/bin/ghostchain

# Create directories
RUN mkdir -p /var/lib/ghostchain/{data,logs,config} && \
    chown -R ghostchain:ghostchain /var/lib/ghostchain

# Copy default configuration
COPY docker/config/ /var/lib/ghostchain/config/

# Switch to ghostchain user
USER ghostchain
WORKDIR /var/lib/ghostchain

# Expose ports
EXPOSE 7777/udp   # QUIC P2P
EXPOSE 8545/tcp   # JSON-RPC
EXPOSE 8443/tcp   # HTTP3/Wraith
EXPOSE 3001/tcp   # WalletD
EXPOSE 9090/udp   # Wallet QUIC

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8545/health || exit 1

# Default command
CMD ["ghostchain", "ghostd", "--production"]