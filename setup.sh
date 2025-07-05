#!/bin/bash
# GhostChain One-liner Setup Script
# Usage: curl -sSL https://raw.githubusercontent.com/yourrepo/ghostchain/main/setup.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GHOSTCHAIN_VERSION=${GHOSTCHAIN_VERSION:-"latest"}
INSTALL_DIR=${INSTALL_DIR:-"/opt/ghostchain"}
DATA_DIR=${DATA_DIR:-"$HOME/.ghostchain"}
SETUP_TYPE=${SETUP_TYPE:-"docker"} # docker, native, or lxc

echo -e "${BLUE}🚀 GhostChain Setup Script${NC}"
echo -e "${BLUE}===========================${NC}"
echo ""

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v lsb_release >/dev/null 2>&1; then
            OS=$(lsb_release -si)
            VERSION=$(lsb_release -sr)
        elif [ -f /etc/os-release ]; then
            . /etc/os-release
            OS=$NAME
            VERSION=$VERSION_ID
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macOS"
        VERSION=$(sw_vers -productVersion)
    else
        print_error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
    
    print_status "Detected OS: $OS $VERSION"
}

# Install dependencies
install_dependencies() {
    print_status "Installing dependencies..."
    
    if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]]; then
        sudo apt-get update
        sudo apt-get install -y curl wget git build-essential pkg-config libssl-dev
        
        if [ "$SETUP_TYPE" = "docker" ]; then
            # Install Docker
            if ! command -v docker >/dev/null 2>&1; then
                curl -fsSL https://get.docker.com | sh
                sudo usermod -aG docker $USER
                print_warning "Please logout and login again to use Docker without sudo"
            fi
            
            # Install Docker Compose
            if ! command -v docker-compose >/dev/null 2>&1; then
                sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
                sudo chmod +x /usr/local/bin/docker-compose
            fi
        fi
        
        if [ "$SETUP_TYPE" = "native" ]; then
            # Install Rust
            if ! command -v rustc >/dev/null 2>&1; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source $HOME/.cargo/env
            fi
        fi
        
    elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"Red Hat"* ]] || [[ "$OS" == *"Fedora"* ]]; then
        sudo yum update -y
        sudo yum install -y curl wget git gcc gcc-c++ make pkgconfig openssl-devel
        
        if [ "$SETUP_TYPE" = "docker" ]; then
            if ! command -v docker >/dev/null 2>&1; then
                curl -fsSL https://get.docker.com | sh
                sudo usermod -aG docker $USER
                sudo systemctl enable docker
                sudo systemctl start docker
            fi
        fi
        
    elif [[ "$OS" == "macOS" ]]; then
        if ! command -v brew >/dev/null 2>&1; then
            print_error "Homebrew is required on macOS. Please install it first."
            exit 1
        fi
        
        brew install curl wget git openssl pkg-config
        
        if [ "$SETUP_TYPE" = "docker" ]; then
            if ! command -v docker >/dev/null 2>&1; then
                print_error "Please install Docker Desktop for Mac"
                exit 1
            fi
        fi
    fi
}

# Setup Docker deployment
setup_docker() {
    print_status "Setting up GhostChain with Docker..."
    
    # Create project directory
    mkdir -p "$INSTALL_DIR"
    cd "$INSTALL_DIR"
    
    # Download docker files
    print_status "Downloading Docker configuration..."
    wget -O docker-compose.yml https://raw.githubusercontent.com/ghostkellz/ghostchain/main/docker-compose.yml
    wget -O Dockerfile https://raw.githubusercontent.com/ghostkellz/ghostchain/main/Dockerfile
    
    mkdir -p docker
    wget -O docker/entrypoint.sh https://raw.githubusercontent.com/ghostkellz/ghostchain/main/docker/entrypoint.sh
    chmod +x docker/entrypoint.sh
    
    # Start services
    print_status "Starting GhostChain services..."
    docker-compose up -d
    
    # Wait for services to start
    print_status "Waiting for services to start..."
    sleep 30
    
    # Check health
    if curl -f http://localhost:8545/health >/dev/null 2>&1; then
        print_status "✅ GhostChain is running successfully!"
        print_status "RPC endpoint: http://localhost:8545"
        print_status "P2P port: 7777"
        print_status "Grafana dashboard: http://localhost:3000 (admin/ghostchain_admin)"
    else
        print_error "❌ GhostChain failed to start. Check logs with: docker-compose logs"
    fi
}

# Setup native installation
setup_native() {
    print_status "Setting up GhostChain natively..."
    
    # Clone repository
    if [ ! -d "$INSTALL_DIR" ]; then
        git clone https://github.com/ghostkellz/ghostchain.git "$INSTALL_DIR"
    fi
    
    cd "$INSTALL_DIR"
    
    # Build
    print_status "Building GhostChain..."
    cargo build --release
    
    # Install binary
    sudo cp target/release/ghostchain /usr/local/bin/
    
    # Create systemd service
    create_systemd_service
    
    # Start service
    sudo systemctl enable ghostchain
    sudo systemctl start ghostchain
    
    print_status "✅ GhostChain installed and started!"
    print_status "Status: sudo systemctl status ghostchain"
    print_status "Logs: sudo journalctl -u ghostchain -f"
}

# Setup LXC container
setup_lxc() {
    print_status "Setting up GhostChain LXC container..."
    
    # Check if LXD is installed
    if ! command -v lxc >/dev/null 2>&1; then
        print_status "Installing LXD..."
        sudo snap install lxd
        sudo lxd init --auto
        sudo usermod -aG lxd $USER
        print_warning "Please logout and login again to use LXC"
        return
    fi
    
    # Create container
    print_status "Creating GhostChain LXC container..."
    lxc launch ubuntu:22.04 ghostchain
    
    # Wait for container to start
    sleep 10
    
    # Setup container
    print_status "Setting up container..."
    lxc exec ghostchain -- apt-get update
    lxc exec ghostchain -- apt-get install -y curl wget git build-essential pkg-config libssl-dev
    
    # Install Rust in container
    lxc exec ghostchain -- curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Clone and build GhostChain
    lxc exec ghostchain -- git clone https://github.com/ghostkellz/ghostchain.git /opt/ghostchain
    lxc exec ghostchain -- bash -c "cd /opt/ghostchain && source ~/.cargo/env && cargo build --release"
    
    # Configure ports
    lxc config device add ghostchain p2p proxy listen=tcp:0.0.0.0:7777 connect=tcp:127.0.0.1:7777
    lxc config device add ghostchain rpc proxy listen=tcp:0.0.0.0:8545 connect=tcp:127.0.0.1:8545
    
    # Start GhostChain
    lxc exec ghostchain -- bash -c "cd /opt/ghostchain && nohup ./target/release/ghostchain node --bind 0.0.0.0:7777 --rpc-port 8545 > /var/log/ghostchain.log 2>&1 &"
    
    print_status "✅ GhostChain LXC container created and started!"
    print_status "Container: lxc exec ghostchain -- bash"
    print_status "Logs: lxc exec ghostchain -- tail -f /var/log/ghostchain.log"
}

# Create systemd service
create_systemd_service() {
    print_status "Creating systemd service..."
    
    sudo tee /etc/systemd/system/ghostchain.service > /dev/null << EOF
[Unit]
Description=GhostChain Node
After=network.target

[Service]
Type=simple
User=ghostchain
Group=ghostchain
ExecStart=/usr/local/bin/ghostchain node --bind 0.0.0.0:7777 --rpc-port 8545 --data-dir $DATA_DIR
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

    # Create ghostchain user
    if ! id "ghostchain" &>/dev/null; then
        sudo useradd -r -s /bin/false -d "$DATA_DIR" ghostchain
        sudo mkdir -p "$DATA_DIR"
        sudo chown ghostchain:ghostchain "$DATA_DIR"
    fi
    
    sudo systemctl daemon-reload
}

# Show usage information
show_usage() {
    echo -e "${BLUE}GhostChain Setup Options:${NC}"
    echo ""
    echo "Environment Variables:"
    echo "  SETUP_TYPE=docker|native|lxc  (default: docker)"
    echo "  INSTALL_DIR=/path/to/install   (default: /opt/ghostchain)"
    echo "  DATA_DIR=/path/to/data         (default: ~/.ghostchain)"
    echo ""
    echo "Examples:"
    echo "  # Docker setup (recommended)"
    echo "  curl -sSL setup.sh | bash"
    echo ""
    echo "  # Native installation"
    echo "  curl -sSL setup.sh | SETUP_TYPE=native bash"
    echo ""
    echo "  # LXC container"
    echo "  curl -sSL setup.sh | SETUP_TYPE=lxc bash"
    echo ""
    echo "  # Custom paths"
    echo "  curl -sSL setup.sh | INSTALL_DIR=/custom/path DATA_DIR=/custom/data bash"
}

# Main setup logic
main() {
    detect_os
    install_dependencies
    
    case "$SETUP_TYPE" in
        "docker")
            setup_docker
            ;;
        "native")
            setup_native
            ;;
        "lxc")
            setup_lxc
            ;;
        *)
            print_error "Unknown setup type: $SETUP_TYPE"
            show_usage
            exit 1
            ;;
    esac
    
    print_status "🎉 GhostChain setup completed!"
    echo ""
    echo "Next steps:"
    echo "1. Test the installation: ghostchain --help"
    echo "2. Start a local testnet: ghostchain chain testnet"
    echo "3. Run integration tests: ghostchain services test-domains"
    echo "4. Check the documentation: https://docs.ghostchain.org"
}

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_usage
            exit 0
            ;;
        --type)
            SETUP_TYPE="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --data-dir)
            DATA_DIR="$2"
            shift 2
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Run main setup
main