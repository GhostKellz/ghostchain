// GhostChain Shared - Common types, crypto, and utilities
//
// This crate contains shared functionality used across the GhostChain ecosystem:
// - Common types and data structures
// - Cryptographic functions (ZCrypto integration)
// - Domain resolution (ENS, Unstoppable, Web5, Ghost)
// - FFI bindings for ZQUIC and GhostBridge integration

pub mod types;
pub mod crypto;
// pub mod transport; // Moved to core Quinn integration

#[cfg(feature = "domains")]
pub mod domains;

#[cfg(feature = "ffi")]
pub mod ffi;

// Re-export commonly used items
pub use types::*;
pub use crypto::*;
// pub use transport::*; // Moved to core Quinn integration

use anyhow::Result;
use tracing::info;

/// Initialize the shared library
pub fn init() -> Result<()> {
    info!("🔧 Initializing GhostChain Shared library");
    info!("✅ GhostChain Shared library initialized");
    Ok(())
}

/// Get shared library version
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}