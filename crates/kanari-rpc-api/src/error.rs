// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::types::ErrorObjectOwned;
use thiserror::Error;

/// RPC API errors
#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Node not ready: {0}")]
    NodeNotReady(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        let (code, message) = match err {
            RpcError::InternalError(msg) => (-32603, format!("Internal error: {}", msg)),
            RpcError::InvalidParams(msg) => (-32602, format!("Invalid params: {}", msg)),
            RpcError::MethodNotFound(msg) => (-32601, format!("Method not found: {}", msg)),
            RpcError::NodeNotReady(msg) => (-32000, format!("Node not ready: {}", msg)),
            RpcError::TransactionFailed(msg) => (-32001, format!("Transaction failed: {}", msg)),
            RpcError::BlockNotFound(msg) => (-32002, format!("Block not found: {}", msg)),
            RpcError::AccountNotFound(msg) => (-32003, format!("Account not found: {}", msg)),
            RpcError::NetworkError(msg) => (-32004, format!("Network error: {}", msg)),
        };

        ErrorObjectOwned::owned(code, message, None::<()>)
    }
}

/// Result type for RPC methods
pub type RpcResult<T> = Result<T, ErrorObjectOwned>;

/// Convert any error to RpcResult
pub fn to_rpc_result<T>(result: Result<T, anyhow::Error>) -> RpcResult<T> {
    result.map_err(|e| RpcError::InternalError(e.to_string()).into())
}
