# Shroud FFI Integration Guide for GhostChain (Rust)

This guide provides complete instructions for integrating Shroud's Zig-based cryptographic and networking libraries with GhostChain's Rust codebase.

## Table of Contents

1. [Overview](#overview)
2. [Build Setup](#build-setup)
3. [FFI Module Reference](#ffi-module-reference)
4. [Rust Integration](#rust-integration)
5. [Usage Examples](#usage-examples)
6. [Error Handling](#error-handling)
7. [Memory Management](#memory-management)
8. [Performance Considerations](#performance-considerations)
9. [Testing](#testing)
10. [Troubleshooting](#troubleshooting)

## Overview

Shroud provides comprehensive FFI bindings across 6 major modules:

- **GhostCipher ZCrypto** - Advanced cryptography with post-quantum support
- **Sigil RealID** - Digital identity management
- **GWallet Core** - Wallet operations and account management
- **GhostBridge** - gRPC over QUIC transport
- **ZCrypto** - Simplified crypto API for Rust integration
- **ZQUIC** - Comprehensive QUIC transport with DNS support

## Build Setup

### 1. Building Shroud with FFI

```bash
# Clone Shroud repository
git clone https://github.com/ghostkellz/shroud.git
cd shroud

# Build all modules with FFI exports
zig build

# Build shared library (for dynamic linking)
zig build -Dshared=true

# Build static library (for static linking)
zig build -Dstatic=true
```

### 2. Rust Project Setup

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core dependencies
libc = "0.2"
tokio = { version = "1.0", features = ["full"] }

[build-dependencies]
bindgen = "0.69"
cc = "1.0"

[lib]
name = "ghostchain_shroud"
crate-type = ["cdylib", "rlib"]
```

### 3. Build Script (`build.rs`)

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    let shroud_path = env::var("SHROUD_PATH")
        .unwrap_or_else(|_| "../shroud".to_string());

    // Link Shroud libraries
    println!("cargo:rustc-link-search=native={}/zig-out/lib", shroud_path);
    println!("cargo:rustc-link-lib=static=shroud");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

### 4. C Header Wrapper (`wrapper.h`)

```c
#ifndef SHROUD_WRAPPER_H
#define SHROUD_WRAPPER_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// CORE TYPES AND CONSTANTS
// ============================================================================

// Result structure for crypto operations
typedef struct {
    bool success;
    uint32_t data_len;
    uint32_t error_code;
} CryptoResult;

// Key sizes
#define ED25519_PUBLIC_KEY_SIZE 32
#define ED25519_PRIVATE_KEY_SIZE 32
#define ED25519_SIGNATURE_SIZE 64
#define SECP256K1_PUBLIC_KEY_SIZE 33
#define SECP256K1_PRIVATE_KEY_SIZE 32
#define SECP256K1_SIGNATURE_SIZE 64
#define BLAKE3_HASH_SIZE 32
#define SHA256_HASH_SIZE 32

// Post-quantum key sizes
#define ML_KEM_768_PUBLIC_KEY_SIZE 1184
#define ML_KEM_768_PRIVATE_KEY_SIZE 2400
#define ML_KEM_768_CIPHERTEXT_SIZE 1088
#define ML_KEM_768_SHARED_SECRET_SIZE 32
#define ML_DSA_65_PUBLIC_KEY_SIZE 1952
#define ML_DSA_65_PRIVATE_KEY_SIZE 4000
#define ML_DSA_65_SIGNATURE_SIZE 3309

// ============================================================================
// ZCRYPTO FFI - SIMPLIFIED CRYPTO API
// ============================================================================

// Error codes
#define ZCRYPTO_SUCCESS 0
#define ZCRYPTO_ERROR_INVALID_INPUT -1
#define ZCRYPTO_ERROR_INVALID_KEY -2
#define ZCRYPTO_ERROR_INVALID_SIGNATURE -3
#define ZCRYPTO_ERROR_BUFFER_TOO_SMALL -4
#define ZCRYPTO_ERROR_INTERNAL -5

// Core cryptographic functions
int zcrypto_ed25519_keypair(uint8_t* public_key, uint8_t* private_key);
int zcrypto_ed25519_sign(const uint8_t* private_key, const uint8_t* message, size_t message_len, uint8_t* signature);
int zcrypto_ed25519_verify(const uint8_t* public_key, const uint8_t* message, size_t message_len, const uint8_t* signature);
int zcrypto_secp256k1_keypair(uint8_t* public_key, uint8_t* private_key);
int zcrypto_secp256k1_sign(const uint8_t* private_key, const uint8_t* message_hash, uint8_t* signature);
int zcrypto_secp256k1_verify(const uint8_t* public_key, const uint8_t* message_hash, const uint8_t* signature);
int zcrypto_blake3_hash(const uint8_t* input, size_t input_len, uint8_t* output);
int zcrypto_sha256_hash(const uint8_t* input, size_t input_len, uint8_t* output);
int zcrypto_random_bytes(uint8_t* buffer, size_t len);
int zcrypto_secure_compare(const uint8_t* a, const uint8_t* b, size_t len);
void zcrypto_secure_zero(uint8_t* buffer, size_t len);
const char* zcrypto_version(void);

// ============================================================================
// GHOSTCIPHER ZCRYPTO FFI - ADVANCED CRYPTO WITH POST-QUANTUM
// ============================================================================

// Advanced cryptographic functions
CryptoResult zcrypto_sha256(const uint8_t* input, uint32_t input_len, uint8_t* output);
CryptoResult zcrypto_blake2b(const uint8_t* input, uint32_t input_len, uint8_t* output);

// Post-quantum cryptography (ML-KEM-768)
CryptoResult zcrypto_ml_kem_768_keygen(uint8_t* public_key, uint8_t* private_key);
CryptoResult zcrypto_ml_kem_768_encaps(const uint8_t* public_key, uint8_t* ciphertext, uint8_t* shared_secret);
CryptoResult zcrypto_ml_kem_768_decaps(const uint8_t* private_key, const uint8_t* ciphertext, uint8_t* shared_secret);

// Post-quantum signatures (ML-DSA-65)
CryptoResult zcrypto_ml_dsa_65_keygen(uint8_t* public_key, uint8_t* private_key);
CryptoResult zcrypto_ml_dsa_65_sign(const uint8_t* private_key, const uint8_t* message, uint32_t message_len, uint8_t* signature);
CryptoResult zcrypto_ml_dsa_65_verify(const uint8_t* public_key, const uint8_t* message, uint32_t message_len, const uint8_t* signature);

// Hybrid cryptography (X25519 + ML-KEM-768)
CryptoResult zcrypto_hybrid_x25519_ml_kem_keygen(uint8_t* classical_public, uint8_t* classical_private,
                                                  uint8_t* pq_public, uint8_t* pq_private);
CryptoResult zcrypto_hybrid_x25519_ml_kem_exchange(const uint8_t* our_classical_private, const uint8_t* our_pq_private,
                                                    const uint8_t* peer_classical_public, const uint8_t* peer_pq_ciphertext,
                                                    uint8_t* shared_secret);

// ============================================================================
// REALID FFI - DIGITAL IDENTITY
// ============================================================================

// RealID types
typedef struct RealIDKeyPair RealIDKeyPair;
typedef struct RealIDPrivateKey RealIDPrivateKey;
typedef struct RealIDPublicKey RealIDPublicKey;
typedef struct RealIDSignature RealIDSignature;
typedef struct QID QID;
typedef struct DeviceFingerprint DeviceFingerprint;

// Error codes
#define REALID_SUCCESS 0
#define REALID_ERROR_INVALID_PASSPHRASE -1
#define REALID_ERROR_INVALID_SIGNATURE -2
#define REALID_ERROR_INVALID_KEY -3
#define REALID_ERROR_CRYPTO -4
#define REALID_ERROR_MEMORY -5
#define REALID_ERROR_BUFFER_TOO_SMALL -6

// RealID functions
int realid_generate_from_passphrase_c(const char* passphrase, size_t passphrase_len, RealIDKeyPair* keypair_out);
int realid_generate_from_passphrase_with_device_c(const char* passphrase, size_t passphrase_len, 
                                                   const DeviceFingerprint* device_fingerprint, RealIDKeyPair* keypair_out);
int realid_sign_c(const uint8_t* data, size_t data_len, const RealIDPrivateKey* private_key, RealIDSignature* signature_out);
int realid_verify_c(const RealIDSignature* signature, const uint8_t* data, size_t data_len, const RealIDPublicKey* public_key);
int realid_qid_from_pubkey_c(const RealIDPublicKey* public_key, QID* qid_out);
int realid_generate_device_fingerprint_c(DeviceFingerprint* fingerprint_out);
int realid_get_public_key_c(const RealIDPrivateKey* private_key, RealIDPublicKey* public_key_out);
int realid_qid_to_string_c(const QID* qid_input, char* buffer, size_t buffer_len, size_t* written_len);

// ============================================================================
// GWALLET FFI - WALLET OPERATIONS
// ============================================================================

// Wallet types
typedef struct {
    void* wallet_ptr;
    void* allocator_ptr;
    bool is_valid;
} GWalletContext;

typedef struct {
    uint8_t address[64];
    uint32_t address_len;
    uint8_t public_key[32];
    uint8_t qid[16];
    uint32_t protocol;
    uint32_t key_type;
} WalletAccount;

typedef struct {
    void* identity_ptr;
    bool is_valid;
} RealIdContext;

typedef struct {
    uint8_t public_key[32];
    uint8_t qid[16];
    bool device_bound;
} ZidIdentity;

typedef struct {
    uint8_t signature[64];
    bool success;
} SignatureResult;

// Error codes
#define FFI_SUCCESS 0
#define FFI_ERROR_INVALID_PARAM -1
#define FFI_ERROR_WALLET_LOCKED -2
#define FFI_ERROR_INSUFFICIENT_FUNDS -3
#define FFI_ERROR_SIGNING_FAILED -4
#define FFI_ERROR_VERIFICATION_FAILED -5
#define FFI_ERROR_MEMORY_ERROR -6
#define FFI_ERROR_INVALID_ADDRESS -7
#define FFI_ERROR_ACCOUNT_NOT_FOUND -8

// Wallet functions
GWalletContext zwallet_init(void);
void zwallet_destroy(GWalletContext* ctx);
int zwallet_create_wallet(GWalletContext* ctx, const char* passphrase, uint32_t passphrase_len,
                          const char* wallet_name, uint32_t wallet_name_len, bool device_bound);
int zwallet_load_wallet(GWalletContext* ctx, const uint8_t* wallet_data, uint32_t data_len,
                        const char* passphrase, uint32_t passphrase_len);
int zwallet_create_account(GWalletContext* ctx, uint32_t protocol, uint32_t key_type, WalletAccount* account_out);
int zwallet_get_balance(GWalletContext* ctx, uint32_t protocol, const char* token, uint32_t token_len, uint64_t* balance_out);
int zwallet_update_balance(GWalletContext* ctx, uint32_t protocol, const char* token, uint32_t token_len, uint64_t amount, uint8_t decimals);
int zwallet_lock(GWalletContext* ctx);
int zwallet_unlock(GWalletContext* ctx, const char* passphrase, uint32_t passphrase_len);
int zwallet_get_master_qid(GWalletContext* ctx, uint8_t qid_out[16]);

// ============================================================================
// ZQUIC FFI - QUIC TRANSPORT
// ============================================================================

// QUIC configuration
typedef struct {
    uint16_t port;
    uint32_t max_connections;
    uint32_t connection_timeout_ms;
    uint8_t enable_ipv6;
    uint8_t tls_verify;
    uint8_t reserved[16];
} ZQuicConfig;

typedef struct {
    uint8_t remote_addr[64];
    uint8_t connection_id[16];
    uint8_t state;
    uint32_t rtt_us;
    uint64_t bytes_sent;
    uint64_t bytes_received;
} ZQuicConnectionInfo;

// QUIC functions
void* zquic_init(const ZQuicConfig* config);
void zquic_destroy(void* ctx);
int zquic_create_server(void* ctx);
int zquic_start_server(void* ctx);
void zquic_stop_server(void* ctx);
void* zquic_create_connection(void* ctx, const char* remote_addr);
void zquic_close_connection(void* conn);
ssize_t zquic_send_data(void* conn, const uint8_t* data, size_t len);
ssize_t zquic_receive_data(void* conn, uint8_t* buffer, size_t max_len);
void* zquic_create_stream(void* conn, uint8_t stream_type);
void zquic_close_stream(void* stream);
ssize_t zquic_stream_send(void* stream, const uint8_t* data, size_t len);
ssize_t zquic_stream_receive(void* stream, uint8_t* buffer, size_t max_len);
int zquic_get_connection_info(void* conn, ZQuicConnectionInfo* info);

// ============================================================================
// GHOSTBRIDGE FFI - gRPC OVER QUIC
// ============================================================================

// GhostBridge types
typedef struct {
    uint16_t port;
    uint32_t max_connections;
    uint32_t request_timeout_ms;
    uint8_t enable_discovery;
    uint8_t reserved[32];
} BridgeConfig;

typedef struct {
    uint8_t service[64];
    uint8_t method[64];
    const uint8_t* data;
    size_t data_len;
    uint64_t request_id;
} GrpcRequest;

typedef struct {
    uint8_t* data;
    size_t data_len;
    uint32_t status;
    uint8_t error_message[256];
    uint64_t response_id;
} GrpcResponse;

// GhostBridge functions
void* ghostbridge_init(const BridgeConfig* config);
void ghostbridge_destroy(void* bridge);
int ghostbridge_start(void* bridge);
void ghostbridge_stop(void* bridge);
int ghostbridge_register_service(void* bridge, const char* name, const char* endpoint);
int ghostbridge_unregister_service(void* bridge, const char* name);
void* ghostbridge_create_grpc_connection(void* bridge, const char* service_name);
void ghostbridge_close_grpc_connection(void* conn);
GrpcResponse* ghostbridge_send_grpc_request(void* conn, const GrpcRequest* request);
void ghostbridge_free_grpc_response(GrpcResponse* response);

#ifdef __cplusplus
}
#endif

#endif // SHROUD_WRAPPER_H
```

## FFI Module Reference

### Quick Reference Table

| Operation | Module | Function | Purpose |
|-----------|---------|----------|---------|
| **Ed25519 Keys** | ZCrypto | `zcrypto_ed25519_keypair()` | Generate Ed25519 keypair |
| **Ed25519 Sign** | ZCrypto | `zcrypto_ed25519_sign()` | Sign with Ed25519 |
| **Ed25519 Verify** | ZCrypto | `zcrypto_ed25519_verify()` | Verify Ed25519 signature |
| **Post-Quantum** | GhostCipher | `zcrypto_ml_kem_768_*()` | Post-quantum key exchange |
| **Identity** | Sigil | `realid_generate_from_passphrase_c()` | Generate RealID identity |
| **Wallet** | GWallet | `zwallet_create_wallet()` | Create wallet |
| **QUIC Transport** | ZQUIC | `zquic_create_connection()` | QUIC connection |
| **gRPC** | GhostBridge | `ghostbridge_send_grpc_request()` | gRPC over QUIC |

## Rust Integration

### 1. Basic FFI Wrapper Module

```rust
// src/ffi.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

// Safe wrappers for common operations
pub struct ShroudCrypto;

impl ShroudCrypto {
    pub fn ed25519_keypair() -> Result<(Vec<u8>, Vec<u8>), i32> {
        let mut public_key = vec![0u8; 32];
        let mut private_key = vec![0u8; 32];
        
        unsafe {
            let result = zcrypto_ed25519_keypair(
                public_key.as_mut_ptr(),
                private_key.as_mut_ptr()
            );
            
            if result == 0 {
                Ok((public_key, private_key))
            } else {
                Err(result)
            }
        }
    }
    
    pub fn ed25519_sign(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>, i32> {
        if private_key.len() != 32 {
            return Err(-1);
        }
        
        let mut signature = vec![0u8; 64];
        
        unsafe {
            let result = zcrypto_ed25519_sign(
                private_key.as_ptr(),
                message.as_ptr(),
                message.len(),
                signature.as_mut_ptr()
            );
            
            if result == 0 {
                Ok(signature)
            } else {
                Err(result)
            }
        }
    }
    
    pub fn ed25519_verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, i32> {
        if public_key.len() != 32 || signature.len() != 64 {
            return Err(-1);
        }
        
        unsafe {
            let result = zcrypto_ed25519_verify(
                public_key.as_ptr(),
                message.as_ptr(),
                message.len(),
                signature.as_ptr()
            );
            
            Ok(result == 0)
        }
    }
    
    pub fn blake3_hash(input: &[u8]) -> Result<Vec<u8>, i32> {
        let mut output = vec![0u8; 32];
        
        unsafe {
            let result = zcrypto_blake3_hash(
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr()
            );
            
            if result == 0 {
                Ok(output)
            } else {
                Err(result)
            }
        }
    }
    
    pub fn random_bytes(len: usize) -> Result<Vec<u8>, i32> {
        let mut buffer = vec![0u8; len];
        
        unsafe {
            let result = zcrypto_random_bytes(buffer.as_mut_ptr(), len);
            
            if result == 0 {
                Ok(buffer)
            } else {
                Err(result)
            }
        }
    }
}

// QUIC Transport wrapper
pub struct ShroudQuic {
    ctx: *mut c_void,
}

impl ShroudQuic {
    pub fn new(port: u16) -> Result<Self, i32> {
        let config = ZQuicConfig {
            port,
            max_connections: 1000,
            connection_timeout_ms: 30000,
            enable_ipv6: 0,
            tls_verify: 1,
            reserved: [0; 16],
        };
        
        unsafe {
            let ctx = zquic_init(&config);
            if ctx.is_null() {
                Err(-1)
            } else {
                Ok(ShroudQuic { ctx })
            }
        }
    }
    
    pub fn create_server(&self) -> Result<(), i32> {
        unsafe {
            let result = zquic_create_server(self.ctx);
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }
    
    pub fn start_server(&self) -> Result<(), i32> {
        unsafe {
            let result = zquic_start_server(self.ctx);
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }
    
    pub fn connect(&self, remote_addr: &str) -> Result<ShroudQuicConnection, i32> {
        let remote_addr_c = CString::new(remote_addr).map_err(|_| -1)?;
        
        unsafe {
            let conn = zquic_create_connection(self.ctx, remote_addr_c.as_ptr());
            if conn.is_null() {
                Err(-1)
            } else {
                Ok(ShroudQuicConnection { conn })
            }
        }
    }
}

impl Drop for ShroudQuic {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                zquic_destroy(self.ctx);
            }
        }
    }
}

pub struct ShroudQuicConnection {
    conn: *mut c_void,
}

impl ShroudQuicConnection {
    pub fn send_data(&self, data: &[u8]) -> Result<usize, i32> {
        unsafe {
            let result = zquic_send_data(self.conn, data.as_ptr(), data.len());
            if result >= 0 {
                Ok(result as usize)
            } else {
                Err(result as i32)
            }
        }
    }
    
    pub fn receive_data(&self, buffer: &mut [u8]) -> Result<usize, i32> {
        unsafe {
            let result = zquic_receive_data(self.conn, buffer.as_mut_ptr(), buffer.len());
            if result >= 0 {
                Ok(result as usize)
            } else {
                Err(result as i32)
            }
        }
    }
}

impl Drop for ShroudQuicConnection {
    fn drop(&mut self) {
        unsafe {
            if !self.conn.is_null() {
                zquic_close_connection(self.conn);
            }
        }
    }
}

// Wallet wrapper
pub struct ShroudWallet {
    ctx: GWalletContext,
}

impl ShroudWallet {
    pub fn new() -> Self {
        unsafe {
            let ctx = zwallet_init();
            ShroudWallet { ctx }
        }
    }
    
    pub fn create_wallet(&mut self, passphrase: &str, wallet_name: &str, device_bound: bool) -> Result<(), i32> {
        unsafe {
            let result = zwallet_create_wallet(
                &mut self.ctx,
                passphrase.as_ptr() as *const c_char,
                passphrase.len() as u32,
                wallet_name.as_ptr() as *const c_char,
                wallet_name.len() as u32,
                device_bound
            );
            
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }
    
    pub fn create_account(&mut self, protocol: u32, key_type: u32) -> Result<WalletAccount, i32> {
        unsafe {
            let mut account = std::mem::zeroed();
            let result = zwallet_create_account(&mut self.ctx, protocol, key_type, &mut account);
            
            if result == 0 {
                Ok(account)
            } else {
                Err(result)
            }
        }
    }
}

impl Drop for ShroudWallet {
    fn drop(&mut self) {
        unsafe {
            zwallet_destroy(&mut self.ctx);
        }
    }
}
```

### 2. Async Rust Integration

```rust
// src/async_transport.rs
use tokio::task;
use std::sync::Arc;
use crate::ffi::ShroudQuic;

pub struct AsyncShroudTransport {
    quic: Arc<ShroudQuic>,
}

impl AsyncShroudTransport {
    pub fn new(port: u16) -> Result<Self, i32> {
        let quic = Arc::new(ShroudQuic::new(port)?);
        Ok(AsyncShroudTransport { quic })
    }
    
    pub async fn start_server(&self) -> Result<(), i32> {
        let quic = self.quic.clone();
        task::spawn_blocking(move || {
            quic.create_server()?;
            quic.start_server()
        }).await.map_err(|_| -1)?
    }
    
    pub async fn connect(&self, remote_addr: String) -> Result<AsyncShroudConnection, i32> {
        let quic = self.quic.clone();
        let conn = task::spawn_blocking(move || {
            quic.connect(&remote_addr)
        }).await.map_err(|_| -1)??;
        
        Ok(AsyncShroudConnection { conn: Arc::new(conn) })
    }
}

pub struct AsyncShroudConnection {
    conn: Arc<crate::ffi::ShroudQuicConnection>,
}

impl AsyncShroudConnection {
    pub async fn send_data(&self, data: Vec<u8>) -> Result<usize, i32> {
        let conn = self.conn.clone();
        task::spawn_blocking(move || {
            conn.send_data(&data)
        }).await.map_err(|_| -1)?
    }
    
    pub async fn receive_data(&self, buffer_size: usize) -> Result<Vec<u8>, i32> {
        let conn = self.conn.clone();
        task::spawn_blocking(move || {
            let mut buffer = vec![0u8; buffer_size];
            let len = conn.receive_data(&mut buffer)?;
            buffer.truncate(len);
            Ok(buffer)
        }).await.map_err(|_| -1)?
    }
}
```

## Usage Examples

### 1. Basic Cryptography

```rust
use shroud_ffi::ShroudCrypto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate Ed25519 keypair
    let (public_key, private_key) = ShroudCrypto::ed25519_keypair()?;
    println!("Generated keypair: pub={:?}, priv={:?}", public_key, private_key);
    
    // Sign a message
    let message = b"Hello, Shroud!";
    let signature = ShroudCrypto::ed25519_sign(&private_key, message)?;
    println!("Signature: {:?}", signature);
    
    // Verify signature
    let is_valid = ShroudCrypto::ed25519_verify(&public_key, message, &signature)?;
    println!("Signature valid: {}", is_valid);
    
    // Hash data
    let hash = ShroudCrypto::blake3_hash(message)?;
    println!("Blake3 hash: {:?}", hash);
    
    // Generate random bytes
    let random_data = ShroudCrypto::random_bytes(32)?;
    println!("Random data: {:?}", random_data);
    
    Ok(())
}
```

### 2. QUIC Transport

```rust
use shroud_ffi::ShroudQuic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create QUIC server
    let server = ShroudQuic::new(8080)?;
    server.create_server()?;
    server.start_server()?;
    println!("QUIC server started on port 8080");
    
    // Create client connection
    let client = ShroudQuic::new(0)?; // 0 = any port
    let conn = client.connect("127.0.0.1:8080")?;
    
    // Send data
    let data = b"Hello from QUIC client!";
    let sent = conn.send_data(data)?;
    println!("Sent {} bytes", sent);
    
    // Receive data
    let mut buffer = vec![0u8; 1024];
    let received = conn.receive_data(&mut buffer)?;
    buffer.truncate(received);
    println!("Received: {:?}", String::from_utf8_lossy(&buffer));
    
    Ok(())
}
```

### 3. Wallet Operations

```rust
use shroud_ffi::ShroudWallet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create wallet
    let mut wallet = ShroudWallet::new();
    wallet.create_wallet("my_secure_passphrase", "test_wallet", false)?;
    
    // Create account
    let account = wallet.create_account(1, 1)?; // protocol=1, key_type=1
    println!("Created account with QID: {:?}", account.qid);
    
    Ok(())
}
```

### 4. Post-Quantum Cryptography

```rust
use shroud_ffi::bindings::*;

fn post_quantum_example() -> Result<(), i32> {
    unsafe {
        // Generate ML-KEM-768 keypair
        let mut public_key = vec![0u8; 1184];
        let mut private_key = vec![0u8; 2400];
        
        let result = zcrypto_ml_kem_768_keygen(
            public_key.as_mut_ptr(),
            private_key.as_mut_ptr()
        );
        
        if result.success {
            println!("Generated post-quantum keypair");
            
            // Key encapsulation
            let mut ciphertext = vec![0u8; 1088];
            let mut shared_secret = vec![0u8; 32];
            
            let encaps_result = zcrypto_ml_kem_768_encaps(
                public_key.as_ptr(),
                ciphertext.as_mut_ptr(),
                shared_secret.as_mut_ptr()
            );
            
            if encaps_result.success {
                println!("Encapsulation successful");
                println!("Shared secret: {:?}", shared_secret);
            }
        }
    }
    
    Ok(())
}
```

### 5. gRPC over QUIC

```rust
use shroud_ffi::bindings::*;

fn grpc_example() -> Result<(), i32> {
    unsafe {
        // Configure GhostBridge
        let config = BridgeConfig {
            port: 50051,
            max_connections: 1000,
            request_timeout_ms: 30000,
            enable_discovery: 1,
            reserved: [0; 32],
        };
        
        // Initialize bridge
        let bridge = ghostbridge_init(&config);
        if bridge.is_null() {
            return Err(-1);
        }
        
        // Start server
        let result = ghostbridge_start(bridge);
        if result != 0 {
            ghostbridge_destroy(bridge);
            return Err(result);
        }
        
        println!("gRPC server started on port 50051");
        
        // Register service
        let service_name = std::ffi::CString::new("greeter").unwrap();
        let endpoint = std::ffi::CString::new("127.0.0.1:50051").unwrap();
        
        ghostbridge_register_service(bridge, service_name.as_ptr(), endpoint.as_ptr());
        
        // Create client connection
        let conn = ghostbridge_create_grpc_connection(bridge, service_name.as_ptr());
        if !conn.is_null() {
            println!("Created gRPC connection");
            
            // Send request
            let request = GrpcRequest {
                service: [0; 64],
                method: [0; 64],
                data: b"Hello".as_ptr(),
                data_len: 5,
                request_id: 1,
            };
            
            let response = ghostbridge_send_grpc_request(conn, &request);
            if !response.is_null() {
                println!("Received response");
                ghostbridge_free_grpc_response(response);
            }
            
            ghostbridge_close_grpc_connection(conn);
        }
        
        // Cleanup
        ghostbridge_stop(bridge);
        ghostbridge_destroy(bridge);
    }
    
    Ok(())
}
```

## Error Handling

### Error Code Mapping

```rust
#[derive(Debug, thiserror::Error)]
pub enum ShroudError {
    #[error("Invalid input parameter")]
    InvalidInput,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Buffer too small")]
    BufferTooSmall,
    #[error("Internal error")]
    Internal,
    #[error("Memory error")]
    Memory,
    #[error("Network error")]
    Network,
    #[error("Crypto operation failed")]
    CryptoFailed,
    #[error("Unknown error: {0}")]
    Unknown(i32),
}

impl From<i32> for ShroudError {
    fn from(code: i32) -> Self {
        match code {
            -1 => ShroudError::InvalidInput,
            -2 => ShroudError::InvalidKey,
            -3 => ShroudError::InvalidSignature,
            -4 => ShroudError::BufferTooSmall,
            -5 => ShroudError::Internal,
            -6 => ShroudError::Memory,
            -7 => ShroudError::Network,
            -8 => ShroudError::CryptoFailed,
            code => ShroudError::Unknown(code),
        }
    }
}

pub type ShroudResult<T> = Result<T, ShroudError>;
```

## Memory Management

### Safe Buffer Management

```rust
use std::ptr;
use std::slice;

pub struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl SafeBuffer {
    pub fn new(size: usize) -> Self {
        let ptr = unsafe { libc::malloc(size) as *mut u8 };
        if ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        
        SafeBuffer {
            ptr,
            len: 0,
            cap: size,
        }
    }
    
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }
    
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn capacity(&self) -> usize {
        self.cap
    }
    
    pub fn set_len(&mut self, len: usize) {
        if len > self.cap {
            panic!("Length exceeds capacity");
        }
        self.len = len;
    }
    
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
    
    pub fn secure_zero(&mut self) {
        unsafe {
            zcrypto_secure_zero(self.ptr, self.cap);
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            self.secure_zero();
            unsafe {
                libc::free(self.ptr as *mut libc::c_void);
            }
        }
    }
}

unsafe impl Send for SafeBuffer {}
unsafe impl Sync for SafeBuffer {}
```

## Performance Considerations

### 1. Zero-Copy Operations

```rust
// Use buffer slices instead of copying data
pub fn sign_message_zero_copy(
    private_key: &[u8; 32],
    message: &[u8],
    signature: &mut [u8; 64]
) -> Result<(), ShroudError> {
    unsafe {
        let result = zcrypto_ed25519_sign(
            private_key.as_ptr(),
            message.as_ptr(),
            message.len(),
            signature.as_mut_ptr()
        );
        
        if result == 0 {
            Ok(())
        } else {
            Err(ShroudError::from(result))
        }
    }
}
```

### 2. Batch Operations

```rust
pub fn batch_hash_blake3(inputs: &[&[u8]]) -> Result<Vec<Vec<u8>>, ShroudError> {
    let mut results = Vec::with_capacity(inputs.len());
    
    for input in inputs {
        let mut output = vec![0u8; 32];
        unsafe {
            let result = zcrypto_blake3_hash(
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr()
            );
            
            if result != 0 {
                return Err(ShroudError::from(result));
            }
        }
        results.push(output);
    }
    
    Ok(results)
}
```

### 3. Connection Pooling

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct QuicConnectionPool {
    connections: Arc<Mutex<Vec<Arc<ShroudQuicConnection>>>>,
    max_connections: usize,
}

impl QuicConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        QuicConnectionPool {
            connections: Arc::new(Mutex::new(Vec::new())),
            max_connections,
        }
    }
    
    pub async fn get_connection(&self) -> Option<Arc<ShroudQuicConnection>> {
        let mut connections = self.connections.lock().await;
        connections.pop()
    }
    
    pub async fn return_connection(&self, conn: Arc<ShroudQuicConnection>) {
        let mut connections = self.connections.lock().await;
        if connections.len() < self.max_connections {
            connections.push(conn);
        }
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ed25519_operations() {
        let (public_key, private_key) = ShroudCrypto::ed25519_keypair().unwrap();
        assert_eq!(public_key.len(), 32);
        assert_eq!(private_key.len(), 32);
        
        let message = b"test message";
        let signature = ShroudCrypto::ed25519_sign(&private_key, message).unwrap();
        assert_eq!(signature.len(), 64);
        
        let is_valid = ShroudCrypto::ed25519_verify(&public_key, message, &signature).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_blake3_hash() {
        let input = b"test input";
        let hash = ShroudCrypto::blake3_hash(input).unwrap();
        assert_eq!(hash.len(), 32);
        
        // Hash should be deterministic
        let hash2 = ShroudCrypto::blake3_hash(input).unwrap();
        assert_eq!(hash, hash2);
    }
    
    #[test]
    fn test_random_bytes() {
        let random1 = ShroudCrypto::random_bytes(32).unwrap();
        let random2 = ShroudCrypto::random_bytes(32).unwrap();
        assert_eq!(random1.len(), 32);
        assert_eq!(random2.len(), 32);
        assert_ne!(random1, random2); // Should be different
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_quic_transport() {
        let server = ShroudQuic::new(8081).unwrap();
        server.create_server().unwrap();
        server.start_server().unwrap();
        
        // Wait for server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let client = ShroudQuic::new(0).unwrap();
        let conn = client.connect("127.0.0.1:8081").unwrap();
        
        let data = b"test data";
        let sent = conn.send_data(data).unwrap();
        assert_eq!(sent, data.len());
    }
}
```

## Troubleshooting

### Common Issues

1. **Build Errors**
   ```bash
   # Make sure Shroud is built first
   cd /path/to/shroud
   zig build
   
   # Check library path
   export SHROUD_PATH=/path/to/shroud
   export LD_LIBRARY_PATH=$SHROUD_PATH/zig-out/lib
   ```

2. **Linking Issues**
   ```toml
   # In Cargo.toml
   [dependencies]
   libc = "0.2"
   
   # In build.rs
   println!("cargo:rustc-link-lib=static=shroud");
   println!("cargo:rustc-link-lib=c");
   ```

3. **Runtime Issues**
   ```rust
   // Check FFI version compatibility
   let version = unsafe { CStr::from_ptr(zcrypto_version()) };
   println!("Shroud version: {:?}", version);
   ```

### Debug Mode

```rust
// Enable debug logging
use log::{debug, info, error};

pub fn debug_ffi_call<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    debug!("Calling FFI function: {}", name);
    let result = f();
    debug!("FFI function {} completed", name);
    result
}

// Usage
let result = debug_ffi_call("zcrypto_ed25519_keypair", || {
    ShroudCrypto::ed25519_keypair()
});
```

### Performance Profiling

```rust
use std::time::Instant;

pub fn profile_crypto_operations() {
    let start = Instant::now();
    
    // Generate 1000 keypairs
    for _ in 0..1000 {
        let _ = ShroudCrypto::ed25519_keypair().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Generated 1000 keypairs in {:?}", duration);
    println!("Average per keypair: {:?}", duration / 1000);
}
```

## Environment Variables

```bash
# Required
export SHROUD_PATH=/path/to/shroud

# Optional
export SHROUD_LOG_LEVEL=debug
export SHROUD_CRYPTO_BACKEND=zcrypto
export SHROUD_QUIC_MAX_CONNECTIONS=1000
export SHROUD_ENABLE_POST_QUANTUM=1
```

This comprehensive guide provides everything needed to integrate Shroud's advanced cryptographic and networking capabilities into GhostChain's Rust codebase. The FFI interface provides access to post-quantum cryptography, high-performance QUIC transport, digital identity management, and wallet operations.