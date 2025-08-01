// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{api::*, error::RpcResult};
use anyhow::Result;
use jsonrpsee::{
    RpcModule,
    core::async_trait,
    server::{ServerBuilder, ServerHandle},
};
use std::{net::SocketAddr, sync::Arc, time::SystemTime, collections::hash_map::DefaultHasher, hash::Hasher, str::FromStr};
use tokio::sync::RwLock;
use tracing::{info, warn};
use kanari_types::{kari_coin::{KARI, DECIMALS}, genesis_config::G_LOCAL_CONFIG};
use move_core_types::u256::U256;
use moveos_types::state::MoveStructType;

/// RPC server configuration
#[derive(Debug, Clone)]
pub struct RpcServerConfig {
    pub listen_address: SocketAddr,
    pub max_connections: u32,
    pub max_request_body_size: u32,
    pub max_response_body_size: u32,
    pub enable_cors: bool,
    pub enable_ws: bool,
    pub batch_requests_limit: u32,
}

impl Default for RpcServerConfig {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1:3031".parse().unwrap(),
            max_connections: 100,
            max_request_body_size: 10 * 1024 * 1024,  // 10MB
            max_response_body_size: 10 * 1024 * 1024, // 10MB
            enable_cors: true,
            enable_ws: true,
            batch_requests_limit: 50,
        }
    }
}

/// Node state for RPC operations
#[derive(Debug, Clone)]
pub struct NodeState {
    pub chain_id: u64,
    pub node_version: String,
    pub node_type: String,
    pub is_syncing: bool,
    pub peer_count: usize,
    pub block_height: u128,
    pub uptime_start: SystemTime,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            chain_id: 1,
            node_version: env!("CARGO_PKG_VERSION").to_string(),
            node_type: "FullNode".to_string(),
            is_syncing: false,
            peer_count: 0,
            block_height: 0,
            uptime_start: SystemTime::now(),
        }
    }
}

/// RPC server implementation
pub struct KanariRpcServer {
    config: RpcServerConfig,
    node_state: Arc<RwLock<NodeState>>,
    server_handle: Option<ServerHandle>,
}

impl Clone for KanariRpcServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            node_state: self.node_state.clone(),
            server_handle: None, // Server handle cannot be cloned
        }
    }
}

impl KanariRpcServer {
    /// Create a new RPC server
    pub fn new(config: RpcServerConfig) -> Self {
        Self {
            config,
            node_state: Arc::new(RwLock::new(NodeState::default())),
            server_handle: None,
        }
    }

    /// Start the RPC server
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting Kanari RPC server on {}",
            self.config.listen_address
        );

        let server = ServerBuilder::default()
            .max_connections(self.config.max_connections)
            .max_request_body_size(self.config.max_request_body_size)
            .max_response_body_size(self.config.max_response_body_size)
            .build(self.config.listen_address)
            .await?;

        let mut module = RpcModule::new(());

        // Create API implementations
        let kanari_impl = KanariRpcImpl::new(self.node_state.clone());
        let admin_impl = AdminRpcImpl::new(self.node_state.clone());
        let debug_impl = DebugRpcImpl::new(self.node_state.clone());

        // Register API methods
        module.merge(kanari_impl.into_rpc())?;
        module.merge(admin_impl.into_rpc())?;
        module.merge(debug_impl.into_rpc())?;

        // Start server
        let handle = server.start(module);
        self.server_handle = Some(handle);

        info!("Kanari RPC server started successfully on http://{}", self.config.listen_address);
        Ok(())
    }

    /// Stop the RPC server
    pub async fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.stop().unwrap();
            info!("Kanari RPC server stopped");
        }
    }

    /// Update node state
    pub async fn update_node_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut NodeState),
    {
        let mut state = self.node_state.write().await;
        updater(&mut *state);
    }

    /// Get server address
    pub fn address(&self) -> SocketAddr {
        self.config.listen_address
    }
    
    /// Get node state for external access
    pub fn get_node_state(&self) -> Arc<RwLock<NodeState>> {
        self.node_state.clone()
    }
}

/// Kanari RPC API implementation
pub struct KanariRpcImpl {
    node_state: Arc<RwLock<NodeState>>,
}

impl KanariRpcImpl {
    pub fn new(node_state: Arc<RwLock<NodeState>>) -> Self {
        Self { node_state }
    }
}

#[async_trait]
impl KanariRpcApiServer for KanariRpcImpl {
    async fn get_node_info(&self) -> RpcResult<NodeInfo> {
        let state = self.node_state.read().await;
        let uptime = SystemTime::now()
            .duration_since(state.uptime_start)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(NodeInfo {
            version: state.node_version.clone(),
            chain_id: state.chain_id,
            node_type: state.node_type.clone(),
            peer_count: state.peer_count,
            block_height: state.block_height,
            is_syncing: state.is_syncing,
            uptime_seconds: uptime,
        })
    }

    async fn get_account(&self, address: String) -> RpcResult<AccountInfo> {
        // TODO: Implement actual account lookup
        warn!("get_account not fully implemented yet");

        Ok(AccountInfo {
            address,
            balance: "0".to_string(),
            sequence_number: 0,
            authentication_key: "0x00".to_string(),
        })
    }

    async fn get_balance(
        &self,
        address: String,
        coin_type: Option<String>,
    ) -> RpcResult<BalanceInfo> {
        // TODO: Implement actual balance lookup
        warn!("get_balance not fully implemented yet");

        let coin_type = coin_type.unwrap_or_else(|| "KARI".to_string());

        Ok(BalanceInfo {
            address,
            coin_type,
            balance: "0".to_string(),
            decimals: 8,
        })
    }

    async fn get_block_by_number(&self, block_number: u128) -> RpcResult<BlockInfo> {
        // TODO: Implement actual block lookup
        warn!("get_block_by_number not fully implemented yet");

        Ok(BlockInfo {
            number: block_number,
            hash: format!("0x{:064x}", block_number),
            parent_hash: format!("0x{:064x}", block_number.saturating_sub(1)),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transaction_count: 0,
            gas_used: 0,
            gas_limit: 1000000,
            state_root: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        })
    }

    async fn get_block_by_hash(&self, _block_hash: String) -> RpcResult<BlockInfo> {
        // TODO: Implement actual block lookup by hash
        warn!("get_block_by_hash not fully implemented yet");

        // For now, just return the latest block
        let state = self.node_state.read().await;
        self.get_block_by_number(state.block_height).await
    }

    async fn get_latest_block(&self) -> RpcResult<BlockInfo> {
        let state = self.node_state.read().await;
        self.get_block_by_number(state.block_height).await
    }

    async fn get_transaction(&self, tx_hash: String) -> RpcResult<TransactionInfo> {
        // TODO: Implement actual transaction lookup
        warn!("get_transaction not fully implemented yet");

        Ok(TransactionInfo {
            hash: tx_hash,
            sender: "0x0000000000000000000000000000000000000000".to_string(),
            recipient: Some("0x0000000000000000000000000000000000000001".to_string()),
            amount: "0".to_string(),
            gas_used: 21000,
            gas_price: 1,
            status: "Pending".to_string(),
            block_number: None,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    async fn send_transaction(&self, _tx_request: TransactionRequest) -> RpcResult<String> {
        // TODO: Implement actual transaction sending
        warn!("send_transaction not fully implemented yet");

        // Generate a more realistic transaction hash
        let mut hasher = DefaultHasher::new();
        hasher.write(format!("tx_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()).as_bytes());
        let tx_hash = format!("0x{:064x}", hasher.finish());

        info!("Transaction submitted: {}", tx_hash);
        Ok(tx_hash)
    }

    async fn get_network_stats(&self) -> RpcResult<NetworkStats> {
        let state = self.node_state.read().await;

        Ok(NetworkStats {
            peer_count: state.peer_count,
            connected_peers: vec![], // TODO: Get actual peer list
            block_height: state.block_height,
            transaction_pool_size: 0, // TODO: Get actual tx pool size
            network_id: state.chain_id.to_string(),
        })
    }

    async fn get_tx_pool_status(&self) -> RpcResult<std::collections::HashMap<String, u64>> {
        // TODO: Implement actual tx pool status
        warn!("get_tx_pool_status not fully implemented yet");

        let mut status = std::collections::HashMap::new();
        status.insert("pending".to_string(), 0);
        status.insert("queued".to_string(), 0);
        Ok(status)
    }

    async fn get_chain_id(&self) -> RpcResult<u64> {
        let state = self.node_state.read().await;
        Ok(state.chain_id)
    }

    async fn get_block_height(&self) -> RpcResult<u128> {
        let state = self.node_state.read().await;
        Ok(state.block_height)
    }

    async fn get_kari_token_info(&self) -> RpcResult<KariTokenInfo> {
        Ok(KariTokenInfo {
            name: "KARI Token".to_string(),
            symbol: "KARI".to_string(),
            decimals: DECIMALS,
            total_supply: "10000000000000000".to_string(), // 100M tokens with 8 decimals
            module_address: KARI::ADDRESS.to_hex_literal(),
            scaling_factor: U256::from(10u64.pow(DECIMALS as u32)).to_string(),
        })
    }

    async fn get_kari_balance(&self, account: Option<String>) -> RpcResult<TokenBalance> {
        // If no account specified, use the Rooch wallet from config
        let rooch_address = match account {
            Some(addr) => addr,
            None => "rooch1u6kv4l8xgdejlvne8728skvx5jugvp2prlhuhglw72xgl82vc5xs8kr9hj".to_string(),
        };
        
        // For demo purposes, return mock balance
        // In real implementation, query the Rooch network for actual balance
        let balance = if rooch_address == "rooch1u6kv4l8xgdejlvne8728skvx5jugvp2prlhuhglw72xgl82vc5xs8kr9hj" {
            100_000_000 * KARI_SCALE // 100M KARI tokens (genesis allocation)
        } else {
            0
        };
        
        Ok(TokenBalance {
            address: rooch_address,
            balance: balance.to_string(),
            decimals: 18,
        })
    }

    async fn get_all_token_balances(&self, address: String) -> RpcResult<Vec<TokenBalance>> {
        // TODO: Implement actual multi-token balance lookup
        warn!("get_all_token_balances not fully implemented yet");
        
        let kari_balance = self.get_kari_balance(address).await?;
        Ok(vec![kari_balance])
    }
000
    async fn get_rooch_wallet_info(&self) -> RpcResult<RoochWalletInfo> {
        let rooch_address = "rooch1u6kv4l8xgdejlvne8728skvx5jugvp2prlhuhglw72xgl82vc5xs8kr9hj".to_string();
        let kari_balance = self.get_kari_balance(rooch_address.clone()).await?;

        Ok(RoochWalletInfo {
            rooch_address: rooch_address.clone(),
            hex_address: "0xe6accafce643732fb2793f94785986a4b88605411fefcba3eef28c8f9d4cc50d".to_string(),
            bitcoin_address: "bcrt1pp44qzxqkf6wy5gpzjy6uzp2zzkldjccrqayssnud24gu2x96gehsjleyq3".to_string(),
            public_key: "0x02be56eda70ca8cfb17cc4139b970e839cc8df1af67a7a721630cc2631f7149261".to_string(),
            is_active: true,
            kari_balance,
        })
    }

    async fn get_rooch_kari_balance(&self) -> RpcResult<TokenBalance> {
        let rooch_address = "rooch1u6kv4l8xgdejlvne8728skvx5jugvp2prlhuhglw72xgl82vc5xs8kr9hj".to_string();
        self.get_kari_balance(rooch_address).await
    }

    async fn get_kanari_dao_info(&self) -> RpcResult<KanariDaoInfo> {
        let genesis_config = &*G_LOCAL_CONFIG;
        let dao_config = &genesis_config.kanari_dao;
        
        // Get DAO Bitcoin address from genesis config
        let dao_bitcoin_address = dao_config.multisign_bitcoin_address.to_string();
        
        // Convert Bitcoin address to Rooch address for balance lookup
        let dao_rooch_address = dao_config.multisign_bitcoin_address.to_rooch_address().to_hex_literal();
        let dao_balance = self.get_kari_balance(dao_rooch_address.clone()).await?;

        Ok(KanariDaoInfo {
            multisign_bitcoin_address: dao_bitcoin_address,
            threshold: dao_config.threshold as u64,
            participant_count: dao_config.participant_public_keys.len(),
            collected_fees: "50000000000000000000000".to_string(), // Mock collected fees
            kari_balance: dao_balance,
        })
    }

    async fn estimate_transaction_fee(&self, tx_request: TransactionRequest) -> RpcResult<TransactionFee> {
        // Calculate fee based on transaction complexity and gas usage
        let base_fee = U256::from(tx_request.gas_limit * tx_request.gas_price);
        let priority_fee = base_fee / U256::from(10u64); // 10% priority fee
        let total_fee = base_fee + priority_fee;
        
        let genesis_config = &*G_LOCAL_CONFIG;
        let dao_address = genesis_config.kanari_dao.multisign_bitcoin_address.to_string();

        Ok(TransactionFee {
            base_fee: base_fee.to_string(),
            priority_fee: priority_fee.to_string(),
            total_fee: total_fee.to_string(),
            fee_recipient: dao_address,
        })
    }

    async fn send_transaction_with_fee(&self, tx_request: TransactionRequest) -> RpcResult<String> {
        // Calculate transaction fee
        let fee_info = self.estimate_transaction_fee(tx_request.clone()).await?;
        
        // Generate transaction hash
        let mut hasher = DefaultHasher::new();
        hasher.write(format!("tx_with_fee_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()).as_bytes());
        let tx_hash = format!("0x{:064x}", hasher.finish());

        info!("Transaction submitted with fee to DAO: {}", tx_hash);
        info!("Transaction details:");
        info!("  From: {}", tx_request.sender);
        info!("  To: {}", tx_request.recipient);
        info!("  Amount: {} KARI", tx_request.amount);
        info!("  Total Fee: {} (sent to DAO: {})", fee_info.total_fee, fee_info.fee_recipient);

        // TODO: Implement actual transaction processing with fee to DAO
        warn!("send_transaction_with_fee not fully implemented yet - fees will be sent to DAO when integrated with blockchain");

        Ok(tx_hash)
    }
}

/// Admin RPC API implementation
pub struct AdminRpcImpl {
    node_state: Arc<RwLock<NodeState>>,
}

impl AdminRpcImpl {
    pub fn new(node_state: Arc<RwLock<NodeState>>) -> Self {
        Self { node_state }
    }
}

#[async_trait]
impl AdminRpcApiServer for AdminRpcImpl {
    async fn add_peer(&self, peer_address: String) -> RpcResult<bool> {
        // TODO: Implement actual peer addition
        warn!("add_peer not fully implemented yet");
        info!("Adding peer: {}", peer_address);
        Ok(true)
    }

    async fn remove_peer(&self, peer_id: String) -> RpcResult<bool> {
        // TODO: Implement actual peer removal
        warn!("remove_peer not fully implemented yet");
        info!("Removing peer: {}", peer_id);
        Ok(true)
    }

    async fn get_peers(&self) -> RpcResult<Vec<String>> {
        // TODO: Implement actual peer list
        warn!("get_peers not fully implemented yet");
        Ok(vec![])
    }

    async fn start_mining(&self) -> RpcResult<bool> {
        // TODO: Implement mining start
        warn!("start_mining not fully implemented yet");
        info!("Starting mining");
        Ok(true)
    }

    async fn stop_mining(&self) -> RpcResult<bool> {
        // TODO: Implement mining stop
        warn!("stop_mining not fully implemented yet");
        info!("Stopping mining");
        Ok(true)
    }

    async fn get_mining_status(&self) -> RpcResult<bool> {
        // TODO: Implement mining status check
        warn!("get_mining_status not fully implemented yet");
        Ok(false)
    }
}

/// Debug RPC API implementation
pub struct DebugRpcImpl {
    node_state: Arc<RwLock<NodeState>>,
}

impl DebugRpcImpl {
    pub fn new(node_state: Arc<RwLock<NodeState>>) -> Self {
        Self { node_state }
    }
}

#[async_trait]
impl DebugRpcApiServer for DebugRpcImpl {
    async fn get_raw_block(&self, block_number: u128) -> RpcResult<String> {
        // TODO: Implement actual raw block retrieval
        warn!("get_raw_block not fully implemented yet");
        Ok(format!("raw_block_{}", block_number))
    }

    async fn get_raw_transaction(&self, tx_hash: String) -> RpcResult<String> {
        // TODO: Implement actual raw transaction retrieval
        warn!("get_raw_transaction not fully implemented yet");
        Ok(format!("raw_tx_{}", tx_hash))
    }

    async fn get_state_at_block(
        &self,
        block_number: u128,
    ) -> RpcResult<std::collections::HashMap<String, String>> {
        // TODO: Implement actual state retrieval
        warn!("get_state_at_block not fully implemented yet");
        let mut state = std::collections::HashMap::new();
        state.insert("block".to_string(), block_number.to_string());
        Ok(state)
    }

    async fn trace_transaction(
        &self,
        tx_hash: String,
    ) -> RpcResult<std::collections::HashMap<String, serde_json::Value>> {
        // TODO: Implement actual transaction tracing
        warn!("trace_transaction not fully implemented yet");
        let mut trace = std::collections::HashMap::new();
        trace.insert("tx_hash".to_string(), serde_json::Value::String(tx_hash));
        Ok(trace)
    }
}
