// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::error::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub version: String,
    pub chain_id: u64,
    pub node_type: String,
    pub peer_count: usize,
    pub block_height: u128,
    pub is_syncing: bool,
    pub uptime_seconds: u64,
}

/// Account information  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: String,
    pub sequence_number: u64,
    pub authentication_key: String,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub sender: String,
    pub recipient: Option<String>,
    pub amount: String,
    pub gas_used: u64,
    pub gas_price: u64,
    pub status: String,
    pub block_number: Option<u128>,
    pub timestamp: u64,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub number: u128,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transaction_count: usize,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub state_root: String,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub peer_count: usize,
    pub connected_peers: Vec<String>,
    pub block_height: u128,
    pub transaction_pool_size: usize,
    pub network_id: String,
}

/// Balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub address: String,
    pub coin_type: String,
    pub balance: String,
    pub decimals: u8,
}

/// KARI Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KariTokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: String,
    pub module_address: String,
    pub scaling_factor: String,
}

/// Token balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub address: String,
    pub balance: String,
    pub balance_scaled: String,
    pub token_info: KariTokenInfo,
}

/// Transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub sender: String,
    pub recipient: String,
    pub amount: String,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Option<String>,
}

/// Main Kanari RPC API trait
#[rpc(server, client, namespace = "kanari")]
pub trait KanariRpcApi {
    /// Get node information
    #[method(name = "getNodeInfo")]
    async fn get_node_info(&self) -> RpcResult<NodeInfo>;

    /// Get account information
    #[method(name = "getAccount")]
    async fn get_account(&self, address: String) -> RpcResult<AccountInfo>;

    /// Get account balance
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        address: String,
        coin_type: Option<String>,
    ) -> RpcResult<BalanceInfo>;

    /// Get block by number
    #[method(name = "getBlockByNumber")]
    async fn get_block_by_number(&self, block_number: u128) -> RpcResult<BlockInfo>;

    /// Get block by hash
    #[method(name = "getBlockByHash")]
    async fn get_block_by_hash(&self, block_hash: String) -> RpcResult<BlockInfo>;

    /// Get latest block
    #[method(name = "getLatestBlock")]
    async fn get_latest_block(&self) -> RpcResult<BlockInfo>;

    /// Get transaction by hash
    #[method(name = "getTransaction")]
    async fn get_transaction(&self, tx_hash: String) -> RpcResult<TransactionInfo>;

    /// Send transaction
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx_request: TransactionRequest) -> RpcResult<String>;

    /// Get network statistics
    #[method(name = "getNetworkStats")]
    async fn get_network_stats(&self) -> RpcResult<NetworkStats>;

    /// Get transaction pool status
    #[method(name = "getTxPoolStatus")]
    async fn get_tx_pool_status(&self) -> RpcResult<HashMap<String, u64>>;

    /// Get chain ID
    #[method(name = "getChainId")]
    async fn get_chain_id(&self) -> RpcResult<u64>;

    /// Get block height
    #[method(name = "getBlockHeight")]
    async fn get_block_height(&self) -> RpcResult<u128>;

    /// Get KARI token information
    #[method(name = "getKariTokenInfo")]
    async fn get_kari_token_info(&self) -> RpcResult<KariTokenInfo>;

    /// Get KARI token balance for an address
    #[method(name = "getKariBalance")]
    async fn get_kari_balance(&self, address: String) -> RpcResult<TokenBalance>;

    /// Get all token balances for an address
    #[method(name = "getAllTokenBalances")]
    async fn get_all_token_balances(&self, address: String) -> RpcResult<Vec<TokenBalance>>;
}

/// Admin RPC API trait
#[rpc(server, client, namespace = "admin")]
pub trait AdminRpcApi {
    /// Add peer
    #[method(name = "addPeer")]
    async fn add_peer(&self, peer_address: String) -> RpcResult<bool>;

    /// Remove peer
    #[method(name = "removePeer")]
    async fn remove_peer(&self, peer_id: String) -> RpcResult<bool>;

    /// Get peers
    #[method(name = "getPeers")]
    async fn get_peers(&self) -> RpcResult<Vec<String>>;

    /// Start mining (for development)
    #[method(name = "startMining")]
    async fn start_mining(&self) -> RpcResult<bool>;

    /// Stop mining (for development)
    #[method(name = "stopMining")]
    async fn stop_mining(&self) -> RpcResult<bool>;

    /// Get mining status
    #[method(name = "getMiningStatus")]
    async fn get_mining_status(&self) -> RpcResult<bool>;
}

/// Debug RPC API trait
#[rpc(server, client, namespace = "debug")]
pub trait DebugRpcApi {
    /// Get raw block
    #[method(name = "getRawBlock")]
    async fn get_raw_block(&self, block_number: u128) -> RpcResult<String>;

    /// Get raw transaction
    #[method(name = "getRawTransaction")]
    async fn get_raw_transaction(&self, tx_hash: String) -> RpcResult<String>;

    /// Get state at block
    #[method(name = "getStateAtBlock")]
    async fn get_state_at_block(&self, block_number: u128) -> RpcResult<HashMap<String, String>>;

    /// Trace transaction
    #[method(name = "traceTransaction")]
    async fn trace_transaction(
        &self,
        tx_hash: String,
    ) -> RpcResult<HashMap<String, serde_json::Value>>;
}

/// Subscription events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionEvent {
    NewBlock(BlockInfo),
    NewTransaction(TransactionInfo),
    PeerConnected(String),
    PeerDisconnected(String),
    NodeStatus(NodeInfo),
}

/// WebSocket subscription API
#[rpc(server, client, namespace = "subscribe")]
pub trait SubscriptionRpcApi {
    /// Subscribe to new blocks
    #[subscription(name = "newBlocks", unsubscribe = "unsubscribeNewBlocks", item = BlockInfo)]
    async fn subscribe_new_blocks(&self) -> jsonrpsee::core::SubscriptionResult;

    /// Subscribe to new transactions
    #[subscription(name = "newTransactions", unsubscribe = "unsubscribeNewTransactions", item = TransactionInfo)]
    async fn subscribe_new_transactions(&self) -> jsonrpsee::core::SubscriptionResult;

    /// Subscribe to peer events
    #[subscription(name = "peerEvents", unsubscribe = "unsubscribePeerEvents", item = String)]
    async fn subscribe_peer_events(&self) -> jsonrpsee::core::SubscriptionResult;

    /// Subscribe to node status
    #[subscription(name = "nodeStatus", unsubscribe = "unsubscribeNodeStatus", item = NodeInfo)]
    async fn subscribe_node_status(&self) -> jsonrpsee::core::SubscriptionResult;
}
