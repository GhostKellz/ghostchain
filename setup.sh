#!/bin/bash
# 🚀 GhostChain One-Liner Setup Script
# Usage: curl -sSL https://raw.githubusercontent.com/ghostkellz/ghostchain/main/setup.sh | bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
GHOSTCHAIN_VERSION="latest"
INSTALL_DIR="$HOME/ghostchain"
COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env"

# Print banner
print_banner() {
    echo -e "${PURPLE}"
    echo "    _____ _               _    _____ _           _       "
    echo "   |  ___| |__   ___  ___| |_ / ____| |__   __ _(_)_ __  "
    echo "   | |_ | '_ \ / _ \/ __| __| |    | '_ \ / _\` | | '_ \ "
    echo "   |  _|| | | | (_) \__ \ |_| |____| | | | (_| | | | | |"
    echo "   |_|  |_| |_|\___/|___/\__|\_____|_| |_|\__,_|_|_| |_|"
    echo "                                                        "
    echo -e "${NC}"
    echo -e "${CYAN}🔗 Pure Zig Blockchain • IPv6 • QUIC • Web5${NC}"
    echo -e "${CYAN}🚀 One-liner setup for instant node deployment${NC}"
    echo ""
}

# Check requirements
check_requirements() {
    echo -e "${BLUE}📋 Checking requirements...${NC}"
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
        echo -e "${YELLOW}💡 Visit: https://docs.docker.com/get-docker/${NC}"
        exit 1
    fi
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        echo -e "${RED}❌ Docker Compose is not installed. Please install Docker Compose first.${NC}"
        echo -e "${YELLOW}💡 Visit: https://docs.docker.com/compose/install/${NC}"
        exit 1
    fi
    
    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}❌ curl is not installed. Please install curl first.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All requirements satisfied${NC}"
}

# Create installation directory
create_install_dir() {
    echo -e "${BLUE}📁 Creating installation directory...${NC}"
    
    if [ -d "$INSTALL_DIR" ]; then
        echo -e "${YELLOW}⚠️  Directory $INSTALL_DIR already exists. Backing up...${NC}"
        mv "$INSTALL_DIR" "${INSTALL_DIR}.backup.$(date +%s)"
    fi
    
    mkdir -p "$INSTALL_DIR"
    cd "$INSTALL_DIR"
    
    echo -e "${GREEN}✅ Created $INSTALL_DIR${NC}"
}

# Download configuration files
download_configs() {
    echo -e "${BLUE}⬇️  Downloading GhostChain configuration...${NC}"
    
    # Download docker-compose.yml
    curl -sSL -o "$COMPOSE_FILE" "https://blockchain.cktechx.com/ghostchain/docker-compose.yml"
    
    # Download additional config files
    mkdir -p docker/{config,nginx,prometheus,grafana}
    
    # Download nginx config
    curl -sSL -o docker/nginx/nginx.conf "https://blockchain.cktechx.com/ghostchain/docker/nginx/nginx.conf"
    
    # Download prometheus config
    curl -sSL -o docker/prometheus/prometheus.yml "https://blockchain.cktechx.com/ghostchain/docker/prometheus/prometheus.yml"
    
    echo -e "${GREEN}✅ Downloaded configuration files${NC}"
}

# Create environment file
create_env_file() {
    echo -e "${BLUE}⚙️  Creating environment configuration...${NC}"
    
    # Get public IP for configuration
    PUBLIC_IP=$(curl -s https://ipv4.icanhazip.com || echo "127.0.0.1")
    
    cat > "$ENV_FILE" << EOF
# GhostChain Configuration
GHOSTCHAIN_VERSION=$GHOSTCHAIN_VERSION
PUBLIC_IP=$PUBLIC_IP
DOMAIN_NAME=localhost

# Network Configuration
GHOSTCHAIN_P2P_PORT=7777
GHOSTCHAIN_RPC_PORT=8545
GHOSTCHAIN_HTTP3_PORT=8443
WALLETD_PORT=3001
ZNS_PORT=8548

# Security
GRAFANA_ADMIN_PASSWORD=ghostchain$(openssl rand -hex 8)
REDIS_PASSWORD=ghostchain$(openssl rand -hex 12)

# Features
ENABLE_MONITORING=true
ENABLE_IPV6=true
ENABLE_METRICS=true
ENABLE_CORS=true

# Deployment
DEPLOYMENT_MODE=production
LOG_LEVEL=info
EOF
    
    echo -e "${GREEN}✅ Created environment configuration${NC}"
    echo -e "${YELLOW}📝 Configuration saved to $ENV_FILE${NC}"
}

# Setup networking
setup_networking() {
    echo -e "${BLUE}🌐 Setting up networking...${NC}"
    
    # Enable IPv6 if not already enabled
    if ! docker network ls | grep -q ghostchain_network; then
        echo -e "${CYAN}📡 Creating IPv6-enabled Docker network...${NC}"
        # Network will be created by docker-compose
    fi
    
    # Check for port conflicts
    local ports=(7777 8545 8443 3001 8548 9091 3000 6379 80 443)
    local conflicts=()
    
    for port in "${ports[@]}"; do
        if netstat -ln 2>/dev/null | grep -q ":$port "; then
            conflicts+=("$port")
        fi
    done
    
    if [ ${#conflicts[@]} -gt 0 ]; then
        echo -e "${YELLOW}⚠️  Port conflicts detected: ${conflicts[*]}${NC}"
        echo -e "${YELLOW}💡 You may need to stop conflicting services or modify ports in $COMPOSE_FILE${NC}"
    fi
    
    echo -e "${GREEN}✅ Network configuration complete${NC}"
}

# Pull and build images
build_images() {
    echo -e "${BLUE}🏗️  Building GhostChain images...${NC}"
    echo -e "${CYAN}📦 This may take a few minutes on first run...${NC}"
    
    # Pull pre-built images and build custom ones
    docker-compose pull --ignore-pull-failures
    docker-compose build --parallel
    
    echo -e "${GREEN}✅ Images built successfully${NC}"
}

# Start services
start_services() {
    echo -e "${BLUE}🚀 Starting GhostChain services...${NC}"
    
    # Start core services first
    echo -e "${CYAN}🔗 Starting blockchain node...${NC}"
    docker-compose up -d ghostd
    
    # Wait for ghostd to be healthy
    echo -e "${CYAN}⏳ Waiting for blockchain node to be ready...${NC}"
    timeout 120 sh -c 'until docker-compose exec ghostd curl -f http://localhost:8545/health >/dev/null 2>&1; do sleep 2; done' || {
        echo -e "${RED}❌ Blockchain node failed to start within 2 minutes${NC}"
        show_logs
        exit 1
    }
    
    # Start remaining services
    echo -e "${CYAN}💼 Starting wallet service...${NC}"
    docker-compose up -d walletd
    
    echo -e "${CYAN}🌍 Starting ZNS resolver...${NC}"
    docker-compose up -d zns-resolver
    
    # Start monitoring if enabled
    if grep -q "ENABLE_MONITORING=true" "$ENV_FILE"; then
        echo -e "${CYAN}📊 Starting monitoring stack...${NC}"
        docker-compose up -d prometheus grafana
    fi
    
    # Start additional services
    echo -e "${CYAN}🔧 Starting additional services...${NC}"
    docker-compose up -d redis nginx
    
    echo -e "${GREEN}✅ All services started successfully${NC}"
}

# Show service status
show_status() {
    echo -e "${BLUE}📊 Service Status:${NC}"
    docker-compose ps
    
    echo ""
    echo -e "${BLUE}🌐 Service URLs:${NC}"
    echo -e "${GREEN}🔗 Blockchain RPC:${NC}    http://localhost:8545"
    echo -e "${GREEN}💼 Wallet API:${NC}       http://localhost:3001"
    echo -e "${GREEN}🌍 ZNS Resolver:${NC}     http://localhost:8548"
    echo -e "${GREEN}📊 Grafana Dashboard:${NC} http://localhost:3000"
    echo -e "${GREEN}📈 Prometheus:${NC}       http://localhost:9091"
    
    echo ""
    echo -e "${BLUE}🔑 Default Credentials:${NC}"
    if [ -f "$ENV_FILE" ]; then
        local grafana_pass=$(grep GRAFANA_ADMIN_PASSWORD "$ENV_FILE" | cut -d'=' -f2)
        echo -e "${CYAN}Grafana:${NC} admin / $grafana_pass"
    fi
}

# Show logs function
show_logs() {
    echo -e "${BLUE}📋 Recent logs:${NC}"
    docker-compose logs --tail=20
}

# Health check
health_check() {
    echo -e "${BLUE}🏥 Running health checks...${NC}"
    
    local services=("ghostd:8545" "walletd:3001" "zns-resolver:8548")
    local healthy=0
    
    for service in "${services[@]}"; do
        local name=$(echo "$service" | cut -d':' -f1)
        local port=$(echo "$service" | cut -d':' -f2)
        
        if curl -sf "http://localhost:$port/health" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ $name is healthy${NC}"
            ((healthy++))
        else
            echo -e "${RED}❌ $name is not responding${NC}"
        fi
    done
    
    echo -e "${BLUE}📊 Health Summary: $healthy/3 services healthy${NC}"
    
    if [ $healthy -eq 3 ]; then
        echo -e "${GREEN}🎉 All core services are running perfectly!${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  Some services may need more time to start${NC}"
        return 1
    fi
}

# Quick start function
quick_start() {
    echo -e "${CYAN}⚡ Quick Start Commands:${NC}"
    echo ""
    echo -e "${YELLOW}# Create a wallet${NC}"
    echo "curl -X POST http://localhost:3001/api/v1/auth/login -d '{\"username\":\"admin\",\"password\":\"admin\"}'"
    echo ""
    echo -e "${YELLOW}# Check blockchain status${NC}"
    echo "curl http://localhost:8545/health"
    echo ""
    echo -e "${YELLOW}# Resolve a .ghost domain${NC}"
    echo "curl http://localhost:8548/api/v1/zns/example.ghost"
    echo ""
    echo -e "${YELLOW}# View logs${NC}"
    echo "docker-compose logs -f ghostd"
    echo ""
    echo -e "${YELLOW}# Stop all services${NC}"
    echo "docker-compose down"
    echo ""
    echo -e "${YELLOW}# Update to latest version${NC}"
    echo "docker-compose pull && docker-compose up -d"
}

# Cleanup function
cleanup() {
    echo -e "${BLUE}🧹 Cleaning up...${NC}"
    docker-compose down --remove-orphans
}

# Main installation function
main() {
    print_banner
    
    echo -e "${BLUE}🚀 Starting GhostChain installation...${NC}"
    echo ""
    
    check_requirements
    create_install_dir
    download_configs
    create_env_file
    setup_networking
    build_images
    start_services
    
    echo ""
    echo -e "${GREEN}🎉 GhostChain installation completed successfully!${NC}"
    echo ""
    
    show_status
    echo ""
    
    # Wait a moment for services to fully start
    echo -e "${CYAN}⏳ Performing final health checks...${NC}"
    sleep 10
    
    if health_check; then
        echo ""
        echo -e "${GREEN}🚀 GhostChain is now running and ready to use!${NC}"
        quick_start
    else
        echo ""
        echo -e "${YELLOW}⚠️  Installation completed but some services need more time${NC}"
        echo -e "${CYAN}💡 Run 'docker-compose logs' to check service status${NC}"
    fi
    
    echo ""
    echo -e "${PURPLE}🔗 Welcome to the GhostChain network!${NC}"
}

# Handle script arguments
case "${1:-install}" in
    "install"|"")
        main
        ;;
    "status")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        show_status
        ;;
    "health")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        health_check
        ;;
    "logs")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        show_logs
        ;;
    "stop")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        docker-compose down
        echo -e "${GREEN}✅ GhostChain stopped${NC}"
        ;;
    "start")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        docker-compose up -d
        echo -e "${GREEN}✅ GhostChain started${NC}"
        ;;
    "restart")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        docker-compose restart
        echo -e "${GREEN}✅ GhostChain restarted${NC}"
        ;;
    "update")
        cd "$INSTALL_DIR" 2>/dev/null || { echo "GhostChain not installed"; exit 1; }
        docker-compose pull
        docker-compose up -d
        echo -e "${GREEN}✅ GhostChain updated${NC}"
        ;;
    "uninstall")
        read -p "Are you sure you want to completely remove GhostChain? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            cd "$INSTALL_DIR" 2>/dev/null && cleanup
            rm -rf "$INSTALL_DIR"
            echo -e "${GREEN}✅ GhostChain uninstalled${NC}"
        fi
        ;;
    "help"|"-h"|"--help")
        echo "GhostChain Setup Script"
        echo ""
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  install    Install and start GhostChain (default)"
        echo "  status     Show service status"
        echo "  health     Run health checks"
        echo "  logs       Show recent logs"
        echo "  start      Start all services"
        echo "  stop       Stop all services"
        echo "  restart    Restart all services"
        echo "  update     Update to latest version"
        echo "  uninstall  Remove GhostChain completely"
        echo "  help       Show this help message"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac