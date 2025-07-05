use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::ffi::{FFIResult, c_to_rust_string, rust_to_c_string};
use crate::types::*;

/// ZQUIC context handle for FFI
pub struct ZQuicContext {
    pub endpoint: String,
    pub config: ZQuicConfig,
    pub connections: Vec<ZQuicConnection>,
}

/// ZQUIC configuration for FFI
#[repr(C)]
pub struct ZQuicConfig {
    pub max_connections: c_uint,
    pub connection_timeout_ms: c_uint,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub cert_path: *const c_char,
    pub key_path: *const c_char,
}

/// ZQUIC connection handle
pub struct ZQuicConnection {
    pub id: String,
    pub remote_addr: String,
    pub status: ConnectionStatus,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum ConnectionStatus {
    Connecting = 0,
    Connected = 1,
    Disconnected = 2,
    Error = 3,
}

/// ZQUIC stream handle for gRPC
pub struct ZQuicGrpcStream {
    pub connection_id: String,
    pub stream_id: u64,
    pub service_name: String,
    pub method_name: String,
}

/// Initialize ZQUIC context
#[unsafe(no_mangle)]
pub extern "C" fn zquic_init(config: *const ZQuicConfig) -> *mut ZQuicContext {
    if config.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let cfg = &*config;
        let cert_path = if cfg.cert_path.is_null() {
            String::new()
        } else {
            c_to_rust_string(cfg.cert_path).unwrap_or_default()
        };
        
        let key_path = if cfg.key_path.is_null() {
            String::new()
        } else {
            c_to_rust_string(cfg.key_path).unwrap_or_default()
        };

        let context = ZQuicContext {
            endpoint: "0.0.0.0:4433".to_string(),
            config: ZQuicConfig {
                max_connections: cfg.max_connections,
                connection_timeout_ms: cfg.connection_timeout_ms,
                enable_compression: cfg.enable_compression,
                enable_encryption: cfg.enable_encryption,
                cert_path: rust_to_c_string(&cert_path),
                key_path: rust_to_c_string(&key_path),
            },
            connections: Vec::new(),
        };

        Box::into_raw(Box::new(context))
    }
}

/// Create a connection to remote endpoint
#[unsafe(no_mangle)]
pub extern "C" fn zquic_create_connection(
    ctx: *mut ZQuicContext,
    remote_addr: *const c_char,
) -> *mut ZQuicConnection {
    if ctx.is_null() || remote_addr.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let context = &mut *ctx;
        let addr = match c_to_rust_string(remote_addr) {
            Ok(addr) => addr,
            Err(_) => return ptr::null_mut(),
        };

        let connection = ZQuicConnection {
            id: uuid::Uuid::new_v4().to_string(),
            remote_addr: addr,
            status: ConnectionStatus::Connecting,
        };

        let conn_ptr = Box::into_raw(Box::new(connection));
        
        // Add to context (in real implementation, we'd manage this properly)
        // context.connections.push((*conn_ptr).clone());
        
        conn_ptr
    }
}

/// Create a gRPC stream over QUIC
#[unsafe(no_mangle)]
pub extern "C" fn zquic_create_grpc_stream(
    connection: *mut ZQuicConnection,
    service_name: *const c_char,
    method_name: *const c_char,
) -> *mut ZQuicGrpcStream {
    if connection.is_null() || service_name.is_null() || method_name.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let conn = &*connection;
        let service = match c_to_rust_string(service_name) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        let method = match c_to_rust_string(method_name) {
            Ok(m) => m,
            Err(_) => return ptr::null_mut(),
        };

        let stream = ZQuicGrpcStream {
            connection_id: conn.id.clone(),
            stream_id: rand::random::<u64>(),
            service_name: service,
            method_name: method,
        };

        Box::into_raw(Box::new(stream))
    }
}

/// Send gRPC data over QUIC stream
#[unsafe(no_mangle)]
pub extern "C" fn zquic_send_grpc_data(
    stream: *mut ZQuicGrpcStream,
    data: *const u8,
    len: usize,
) -> c_int {
    if stream.is_null() || data.is_null() || len == 0 {
        return -1;
    }

    unsafe {
        let _stream = &*stream;
        let _data_slice = std::slice::from_raw_parts(data, len);
        
        // In real implementation, this would send data over QUIC
        // For now, we simulate success
        0
    }
}

/// Receive gRPC data from QUIC stream
#[unsafe(no_mangle)]
pub extern "C" fn zquic_recv_grpc_data(
    stream: *mut ZQuicGrpcStream,
    buffer: *mut u8,
    buffer_len: usize,
    received_len: *mut usize,
) -> c_int {
    if stream.is_null() || buffer.is_null() || buffer_len == 0 || received_len.is_null() {
        return -1;
    }

    unsafe {
        let _stream = &*stream;
        
        // In real implementation, this would receive data from QUIC
        // For now, we simulate receiving empty data
        *received_len = 0;
        0
    }
}

/// Get connection status
#[unsafe(no_mangle)]
pub extern "C" fn zquic_connection_status(connection: *mut ZQuicConnection) -> ConnectionStatus {
    if connection.is_null() {
        return ConnectionStatus::Error;
    }

    unsafe {
        (*connection).status
    }
}

/// Close gRPC stream
#[unsafe(no_mangle)]
pub extern "C" fn zquic_close_grpc_stream(stream: *mut ZQuicGrpcStream) {
    if !stream.is_null() {
        unsafe {
            Box::from_raw(stream);
        }
    }
}

/// Close connection
#[unsafe(no_mangle)]
pub extern "C" fn zquic_close_connection(connection: *mut ZQuicConnection) {
    if !connection.is_null() {
        unsafe {
            Box::from_raw(connection);
        }
    }
}

/// Destroy ZQUIC context
#[unsafe(no_mangle)]
pub extern "C" fn zquic_destroy(ctx: *mut ZQuicContext) {
    if !ctx.is_null() {
        unsafe {
            let context = Box::from_raw(ctx);
            // Clean up cert_path and key_path C strings
            if !context.config.cert_path.is_null() {
                CString::from_raw(context.config.cert_path as *mut c_char);
            }
            if !context.config.key_path.is_null() {
                CString::from_raw(context.config.key_path as *mut c_char);
            }
        }
    }
}

/// Get last error message (thread-local)
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = std::cell::RefCell::new(None);
}

#[unsafe(no_mangle)]
pub extern "C" fn zquic_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null())
    })
}

fn set_last_error(error: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(error).ok();
    });
}