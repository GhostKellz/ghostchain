#!/bin/bash
# GhostChain Development Environment Startup Script

set -e

echo "🚀 Starting GhostChain Development Environment..."

# Create config directory if it doesn't exist
mkdir -p config

# Create development config files if they don't exist
if [ ! -f config/ghostd-dev.toml ]; then
    echo "Creating default ghostd development config..."
    cat > config/ghostd-dev.toml << EOF
[chain]
chain_id = "ghostchain-dev"
block_time_ms = 2000
epoch_length = 10
enable_contracts = true
enable_mining = true
enable_domains = true

[network]
bind_address = "0.0.0.0:8545"
enable_ipv6 = true
ghostbridge_enabled = true

[rpc]
enabled = true
bind_address = "0.0.0.0:8547"

[zquic]
enabled = true
bind_address = "0.0.0.0:8549"

[logging]
level = "debug"
EOF
fi

if [ ! -f config/walletd-dev.toml ]; then
    echo "Creating default walletd development config..."
    cat > config/walletd-dev.toml << EOF
[wallet]
default_algorithm = "ed25519"
enable_hd_wallets = true

[network]
ghostd_endpoint = "http://ghostd-dev:8547"
timeout_seconds = 30

[api]
enabled = true
bind_address = "0.0.0.0:8548"

[zquic]
enabled = true
bind_address = "0.0.0.0:8550"

[logging]
level = "debug"
EOF
fi

# Start development environment
echo "Starting Docker Compose development environment..."
docker-compose -f docker-compose.dev.yml up --build

echo "✅ Development environment started!"
echo ""
echo "🔗 Services available at:"
echo "   • GhostD RPC: http://localhost:8545"
echo "   • GhostD API: http://localhost:8547"
echo "   • WalletD API: http://localhost:8548"
echo "   • Redis: localhost:6379"
echo ""
echo "📋 Useful commands:"
echo "   • Check ghostd status: curl http://localhost:8547/api/v1/status"
echo "   • Check walletd status: curl http://localhost:8548/health"
echo "   • View logs: docker-compose -f docker-compose.dev.yml logs -f [service]"