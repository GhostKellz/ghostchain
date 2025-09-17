#!/bin/bash
# GhostChain Docker Build Script

set -e

echo "🏗️  Building GhostChain Docker images..."

# Build both ghostd and walletd images
echo "Building ghostd image..."
docker build --target ghostd -t ghostchain/ghostd:latest .

echo "Building walletd image..."
docker build --target walletd -t ghostchain/walletd:latest .

# Tag with version if provided
if [ ! -z "$1" ]; then
    VERSION=$1
    echo "Tagging with version: $VERSION"
    docker tag ghostchain/ghostd:latest ghostchain/ghostd:$VERSION
    docker tag ghostchain/walletd:latest ghostchain/walletd:$VERSION
fi

echo "✅ Docker images built successfully"
echo "Available images:"
docker images | grep ghostchain