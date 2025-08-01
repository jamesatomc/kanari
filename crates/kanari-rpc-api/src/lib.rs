// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;

// Re-export important types
pub use jsonrpsee;
pub use kanari_types::*;

pub mod api;
pub mod error;
pub mod server;

pub use api::*;
pub use error::*;
pub use server::*;

/// RPC API version
pub const RPC_API_VERSION: &str = "1.0.0";

/// Initialize the RPC API module
pub fn init_rpc_api() -> Result<()> {
    tracing::info!("Initializing Kanari RPC API module v{}", RPC_API_VERSION);
    Ok(())
}
