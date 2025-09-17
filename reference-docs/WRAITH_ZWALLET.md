# Zwallet + Wraith Integration Guide

## Overview

This document outlines the integration between Zwallet (secure multi-protocol wallet) and Wraith (high-performance HTTP/3 reverse proxy and static server). The integration creates a powerful Web3 gateway that combines Zwallet's blockchain capabilities with Wraith's modern web serving features.

## Architecture Benefits

### Why Wraith + Zwallet?

1. **HTTP/3 Performance**: Wraith's QUIC-based HTTP/3 support provides:
   - Reduced connection overhead
   - Multiplexed streams without head-of-line blocking
   - Better performance over lossy networks
   - Faster connection establishment

2. **Security**: Wraith's security features enhance Zwallet's Web3 bridge:
   - TLS 1.3 mandatory minimum
   - Automatic certificate management
   - Built-in rate limiting and DDoS protection
   - CSRF, HSTS, and CSP headers

3. **Modern Web Standards**: 
   - Brotli/Gzip compression
   - WebSocket support for real-time updates
   - Static file serving for wallet frontends
   - Advanced routing and middleware

## Integration Components

### 1. Enhanced Bridge Server (`wraith_bridge.zig`)

```zig
// High-level integration example
var server = try wraith.WraithServer.init(allocator, .{
    .bind_address = "::",
    .port = 8443,
    .enable_http3 = true,
    .tls = .{
        .auto_cert = true,
        .min_version = .tls13,
    },
});

// Add Zwallet Web3 API routes
try server.router.addRoute(.{
    .path = "/api/v1/*",
    .method = .POST,
    .handler = handleWeb3Request,
    .middleware = &[_]wraith.Middleware{
        wraith.middleware.cors(.{}),
        wraith.middleware.rateLimit(.{}),
        wraith.middleware.auth(.bearer_token),
    },
});
```

### 2. API Endpoints with Wraith Features

| Endpoint | Method | Features |
|----------|--------|----------|
| `/api/v1/wallet/create` | POST | Rate limiting, CORS, validation |
| `/api/v1/transaction/send` | POST | Auth, JSON validation, logging |
| `/api/v1/identity/resolve/*` | GET | Caching, compression |
| `/ws/transactions` | WebSocket | Real-time updates |
| `/` | Static | SPA frontend serving |

### 3. Security Enhancements

```zig
.security = .{
    .enable_hsts = true,
    .enable_csp = true,
    .rate_limiting = .{
        .requests_per_minute = 100,
        .burst_size = 10,
    },
    .auth = .{
        .require_bearer_token = true,
        .jwt_validation = true,
    },
},
```

## Use Cases

### 1. Web3 DApp Gateway

Zwallet + Wraith can serve as a secure gateway for Web3 DApps:

```
DApp Frontend (React/Vue) 
    ↓ (HTTP/3, WSS)
Wraith Server
    ↓ (Internal API)
Zwallet Core
    ↓ (RPC)
Blockchain Networks
```

### 2. Wallet-as-a-Service

Provide wallet functionality via a secure API:

- **Multi-tenant**: Serve multiple wallet instances
- **Rate limited**: Prevent abuse
- **Cached**: Fast identity resolution
- **Real-time**: WebSocket transaction updates

### 3. Mobile/Desktop Wallet Backend

Wraith can serve as the backend for mobile/desktop wallet apps:

```
Mobile App
    ↓ (HTTP/3 API)
Wraith + Zwallet
    ↓ (Blockchain RPC)
Ethereum, Bitcoin, etc.
```

## Configuration

### Wraith-specific Configuration

```zig
const config = WraithConfig{
    .port = 8443,
    .enable_http3 = true,
    .enable_auto_cert = true,
    .cors_origins = &[_][]const u8{
        "https://wallet.app",
        "https://localhost:3000",
    },
    .rate_limit_rpm = 100,
    .cache_ttl_seconds = 300,
};
```

### Security Configuration

```zig
const security = WraithConfig.Security{
    .require_auth = true,
    .api_key_header = "X-API-Key",
    .jwt_secret = "your-jwt-secret",
    .enable_csrf = true,
};
```

## Performance Optimizations

### 1. Caching Strategy

- **Identity Resolution**: Cache ENS/Unstoppable domain lookups
- **Transaction Status**: Cache recent transaction statuses
- **Static Assets**: Serve wallet frontend with aggressive caching

### 2. Connection Management

- **HTTP/3**: Reduced connection overhead
- **Connection Pooling**: Reuse blockchain RPC connections
- **Compression**: Brotli for API responses

### 3. Resource Limits

```zig
const performance = WraithConfig.Performance{
    .max_request_size = 1024 * 1024, // 1MB
    .connection_timeout = 30, // seconds
    .max_concurrent_connections = 1000,
    .enable_compression = true,
};
```

## Real-time Features

### WebSocket Support

Wraith's WebSocket support enables real-time wallet features:

```zig
// Transaction status updates
try server.router.addWebSocket(.{
    .path = "/ws/transactions",
    .handler = handleTransactionUpdates,
    .auth_required = true,
});

// Price feed updates
try server.router.addWebSocket(.{
    .path = "/ws/prices",
    .handler = handlePriceUpdates,
    .rate_limit = .{ .messages_per_minute = 60 },
});
```

## Deployment Scenarios

### 1. Development Setup

```bash
# Start Zwallet with Wraith bridge
zig build run -- bridge --port 8443 --enable-http3

# Access via HTTP/3
curl --http3 https://localhost:8443/api/v1/wallet/status
```

### 2. Production Deployment

```yaml
# Docker Compose example
version: '3.8'
services:
  zwallet-wraith:
    build: .
    ports:
      - "443:8443"
    environment:
      - ZWALLET_MODE=bridge
      - WRAITH_ENABLE_HTTP3=true
      - WRAITH_AUTO_CERT=true
    volumes:
      - ./certs:/app/certs
      - ./wallet-data:/app/data
```

### 3. Load Balancing

Wraith can be deployed behind a load balancer:

```
Load Balancer (HAProxy/Nginx)
    ↓
Multiple Wraith + Zwallet instances
    ↓
Shared database for wallet state
```

## Monitoring and Observability

### Metrics Integration

Wraith provides built-in metrics that can be exposed:

```zig
// Prometheus metrics endpoint
try server.router.addRoute(.{
    .path = "/metrics",
    .method = .GET,
    .handler = wraith.handlers.prometheus,
    .middleware = &[_]wraith.Middleware{
        wraith.middleware.basicAuth(.{
            .username = "admin",
            .password = "secret",
        }),
    },
});
```

### Health Checks

```zig
try server.router.addRoute(.{
    .path = "/health",
    .method = .GET,
    .handler = handleHealthCheck,
    .cache = .{ .ttl_seconds = 30 },
});
```

## Future Enhancements

### 1. Advanced Features

- **GraphQL API**: Alternative to REST for complex queries
- **gRPC Support**: For high-performance internal services
- **WebAssembly**: Run wallet logic in browser via WASM

### 2. Blockchain-specific Optimizations

- **EIP-1559**: Gas fee optimization for Ethereum
- **Layer 2**: Polygon, Arbitrum, Optimism support
- **Cross-chain**: Atomic swaps and bridges

### 3. Enterprise Features

- **Multi-tenancy**: Isolated wallet instances
- **Audit Logging**: Comprehensive transaction logs
- **Compliance**: KYC/AML integration hooks

## Getting Started

1. **Install Dependencies**: Ensure Wraith and Zwallet are available
2. **Configure**: Set up `WraithConfig` for your environment
3. **Deploy**: Use the enhanced bridge server
4. **Test**: Verify HTTP/3 connectivity and API functionality
5. **Monitor**: Set up observability and alerting

This integration creates a production-ready, high-performance Web3 gateway that leverages the best of both Zwallet's blockchain expertise and Wraith's modern web serving capabilities.
