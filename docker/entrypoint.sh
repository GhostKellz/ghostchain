#!/bin/bash
set -e

# GhostChain Docker Entrypoint Script
echo "🚀 Starting GhostChain..."

# Set default environment variables
export GHOSTCHAIN_DATA_DIR=${GHOSTCHAIN_DATA_DIR:-"/opt/ghostchain/data"}
export GHOSTCHAIN_LOG_LEVEL=${GHOSTCHAIN_LOG_LEVEL:-"info"}
export GHOSTCHAIN_CHAIN_ID=${GHOSTCHAIN_CHAIN_ID:-"ghostchain-docker"}
export GHOSTCHAIN_RPC_PORT=${GHOSTCHAIN_RPC_PORT:-"8545"}
export GHOSTCHAIN_P2P_PORT=${GHOSTCHAIN_P2P_PORT:-"7777"}

# Create directories if they don't exist
mkdir -p "$GHOSTCHAIN_DATA_DIR"
mkdir -p /opt/ghostchain/logs
mkdir -p /opt/ghostchain/certs

# Generate self-signed certificates if they don't exist
if [ ! -f /opt/ghostchain/certs/ghostchain.crt ]; then
    echo "🔐 Generating self-signed certificates..."
    openssl req -x509 -newkey rsa:4096 -keyout /opt/ghostchain/certs/ghostchain.key \
        -out /opt/ghostchain/certs/ghostchain.crt -days 365 -nodes \
        -subj "/C=US/ST=Blockchain/L=GhostChain/O=GhostChain/CN=localhost"
    chmod 600 /opt/ghostchain/certs/ghostchain.key
fi

# Function to start different services
start_service() {
    case "$1" in
        "node")
            echo "🏗️  Starting GhostChain node..."
            exec ghostchain node \
                --bind "0.0.0.0:${GHOSTCHAIN_P2P_PORT}" \
                --rpc-port "${GHOSTCHAIN_RPC_PORT}" \
                --chain-id "${GHOSTCHAIN_CHAIN_ID}" \
                --data-dir "${GHOSTCHAIN_DATA_DIR}" \
                "${@:2}"
            ;;
        "testnet")
            echo "🧪 Starting local testnet..."
            exec ghostchain chain testnet --action start
            ;;
        "rpc")
            echo "🌐 Starting RPC server only..."
            exec ghostchain rpc --bind "0.0.0.0:${GHOSTCHAIN_RPC_PORT}"
            ;;
        "cli")
            echo "💻 Starting CLI mode..."
            exec ghostchain "${@:2}"
            ;;
        "test")
            echo "🧪 Running integration tests..."
            ghostchain services test-zquic
            ghostchain services test-ghostbridge
            ghostchain services test-domains
            ghostchain services test-tokens
            ;;
        *)
            echo "❌ Unknown service: $1"
            echo "Available services: node, testnet, rpc, cli, test"
            exit 1
            ;;
    esac
}

# Handle different startup modes
if [ "$1" = "node" ] || [ "$1" = "testnet" ] || [ "$1" = "rpc" ] || [ "$1" = "cli" ] || [ "$1" = "test" ]; then
    start_service "$@"
else
    # Default: start node
    start_service "node" "$@"
fi