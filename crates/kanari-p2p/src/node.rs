// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::message::{Message, MessageType, NodeInfoPayload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Unique node identifier
pub type NodeId = String;

/// Node information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub version: String,
    pub chain_id: u64,
    pub node_type: NodeType,
    pub listening_addresses: Vec<String>,
    pub capabilities: Vec<String>,
    pub joined_at: u64,
    pub last_seen: u64,
    pub initial_balance: u64, // Add initial balance with default 100000
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Validator,
    FullNode,
    LightNode,
    Bootstrap,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::FullNode
    }
}

/// Main node structure
#[derive(Debug)]
pub struct Node {
    pub info: NodeInfo,
    pub is_running: bool,
    pub connected_peers: HashMap<NodeId, SystemTime>,
    pub message_history: Vec<Message>,
}

impl Node {
    /// Create a new node with default initial balance of 100000
    pub fn new(name: String, chain_id: u64) -> Self {
        let node_id = Uuid::new_v4().to_string();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = NodeInfo {
            id: node_id.clone(),
            name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            chain_id,
            node_type: NodeType::default(),
            listening_addresses: vec![],
            capabilities: vec![
                "block_validation".to_string(),
                "transaction_processing".to_string(),
                "consensus_participation".to_string(),
            ],
            joined_at: current_time,
            last_seen: current_time,
            initial_balance: 100000, // Default initial balance as requested
        };

        Self {
            info,
            is_running: false,
            connected_peers: HashMap::new(),
            message_history: Vec::new(),
        }
    }

    /// Create node with custom initial balance
    pub fn new_with_balance(name: String, chain_id: u64, initial_balance: u64) -> Self {
        let mut node = Self::new(name, chain_id);
        node.info.initial_balance = initial_balance;
        node
    }

    /// Create a validator node
    pub fn new_validator(name: String, chain_id: u64) -> Self {
        let mut node = Self::new(name, chain_id);
        node.info.node_type = NodeType::Validator;
        node.info.capabilities.push("block_production".to_string());
        node
    }

    /// Create a bootstrap node
    pub fn new_bootstrap(name: String, chain_id: u64) -> Self {
        let mut node = Self::new(name, chain_id);
        node.info.node_type = NodeType::Bootstrap;
        node.info.capabilities.push("peer_discovery".to_string());
        node
    }

    /// Start the node
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.is_running {
            anyhow::bail!("Node is already running");
        }

        tracing::info!("Starting node: {} (ID: {})", self.info.name, self.info.id);
        tracing::info!("Node type: {:?}", self.info.node_type);
        tracing::info!("Initial balance: {} KARI", self.info.initial_balance);
        tracing::info!("Chain ID: {}", self.info.chain_id);

        self.is_running = true;
        self.update_last_seen();

        // Announce node to network
        self.announce_to_network()?;

        Ok(())
    }

    /// Stop the node
    pub fn stop(&mut self) -> anyhow::Result<()> {
        if !self.is_running {
            anyhow::bail!("Node is not running");
        }

        tracing::info!("Stopping node: {}", self.info.name);
        self.is_running = false;
        self.connected_peers.clear();

        Ok(())
    }

    /// Connect to another node
    pub fn connect_to_peer(&mut self, peer_id: NodeId) -> anyhow::Result<()> {
        if self.connected_peers.contains_key(&peer_id) {
            tracing::warn!("Already connected to peer: {}", peer_id);
            return Ok(());
        }

        tracing::info!("Connecting to peer: {}", peer_id);
        self.connected_peers
            .insert(peer_id.clone(), SystemTime::now());

        // Send connection message
        let message = self.create_connection_message(&peer_id)?;
        self.message_history.push(message);

        Ok(())
    }

    /// Disconnect from a peer
    pub fn disconnect_from_peer(&mut self, peer_id: &NodeId) -> anyhow::Result<()> {
        if let Some(_) = self.connected_peers.remove(peer_id) {
            tracing::info!("Disconnected from peer: {}", peer_id);
        } else {
            tracing::warn!("Peer not found in connected peers: {}", peer_id);
        }
        Ok(())
    }

    /// Send a message
    pub fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        if !self.is_running {
            anyhow::bail!("Node is not running");
        }

        tracing::debug!("Sending message: {:?}", message.msg_type);
        self.message_history.push(message);
        Ok(())
    }

    /// Process received message
    pub fn process_message(&mut self, message: Message) -> anyhow::Result<()> {
        if !self.is_running {
            anyhow::bail!("Node is not running");
        }

        tracing::debug!(
            "Processing message: {:?} from {:?}",
            message.msg_type,
            message.sender
        );

        match message.msg_type {
            MessageType::NodeJoin => self.handle_node_join(message)?,
            MessageType::NodeLeave => self.handle_node_leave(message)?,
            MessageType::NodeHeartbeat => self.handle_heartbeat(message)?,
            MessageType::BlockProposal => self.handle_block_proposal(message)?,
            MessageType::TransactionBroadcast => self.handle_transaction(message)?,
            _ => {
                tracing::debug!("Unhandled message type: {:?}", message.msg_type);
            }
        }

        self.message_history.push(message);
        Ok(())
    }

    /// Get node statistics
    pub fn get_stats(&self) -> NodeStats {
        NodeStats {
            node_id: self.info.id.clone(),
            connected_peers: self.connected_peers.len(),
            messages_processed: self.message_history.len(),
            uptime_seconds: if self.is_running {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    - self.info.joined_at
            } else {
                0
            },
            initial_balance: self.info.initial_balance,
        }
    }

    /// Update last seen timestamp
    fn update_last_seen(&mut self) {
        self.info.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Announce node to network
    fn announce_to_network(&mut self) -> anyhow::Result<()> {
        let payload = NodeInfoPayload {
            node_id: self.info.id.clone(),
            node_type: format!("{:?}", self.info.node_type),
            version: self.info.version.clone(),
            chain_id: self.info.chain_id,
            listening_addresses: self.info.listening_addresses.clone(),
            capabilities: self.info.capabilities.clone(),
            initial_balance: self.info.initial_balance,
        };

        let payload_bytes = serde_json::to_vec(&payload)?;
        let message =
            Message::new(MessageType::NodeJoin, payload_bytes).with_sender(self.info.id.clone());

        self.message_history.push(message);
        Ok(())
    }

    /// Create connection message
    fn create_connection_message(&self, target_peer: &NodeId) -> anyhow::Result<Message> {
        let payload = serde_json::to_vec(&format!("Connection request from {}", self.info.id))?;
        let message = Message::new(MessageType::PeerConnection, payload)
            .with_sender(self.info.id.clone())
            .with_target(target_peer.clone());
        Ok(message)
    }

    /// Handle node join message
    fn handle_node_join(&mut self, message: Message) -> anyhow::Result<()> {
        if let Some(sender) = &message.sender {
            tracing::info!("Node joined network: {}", sender);
            // Could add to peer list or perform other join logic
        }
        Ok(())
    }

    /// Handle node leave message
    fn handle_node_leave(&mut self, message: Message) -> anyhow::Result<()> {
        if let Some(sender) = &message.sender {
            tracing::info!("Node left network: {}", sender);
            self.connected_peers.remove(sender);
        }
        Ok(())
    }

    /// Handle heartbeat message
    fn handle_heartbeat(&mut self, message: Message) -> anyhow::Result<()> {
        if let Some(sender) = &message.sender {
            self.connected_peers
                .insert(sender.clone(), SystemTime::now());
        }
        Ok(())
    }

    /// Handle block proposal
    fn handle_block_proposal(&mut self, _message: Message) -> anyhow::Result<()> {
        // TODO: Implement block validation and voting logic
        tracing::debug!("Processing block proposal");
        Ok(())
    }

    /// Handle transaction
    fn handle_transaction(&mut self, _message: Message) -> anyhow::Result<()> {
        // TODO: Implement transaction validation and processing
        tracing::debug!("Processing transaction");
        Ok(())
    }
}

/// Node statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub node_id: NodeId,
    pub connected_peers: usize,
    pub messages_processed: usize,
    pub uptime_seconds: u64,
    pub initial_balance: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("test-node".to_string(), 3);
        assert_eq!(node.info.name, "test-node");
        assert_eq!(node.info.chain_id, 3);
        assert_eq!(node.info.initial_balance, 100000);
        assert!(!node.is_running);
    }

    #[test]
    fn test_node_with_custom_balance() {
        let node = Node::new_with_balance("test-node".to_string(), 3, 50000);
        assert_eq!(node.info.initial_balance, 50000);
    }

    #[test]
    fn test_validator_node() {
        let node = Node::new_validator("validator-node".to_string(), 3);
        assert_eq!(node.info.node_type, NodeType::Validator);
        assert!(node
            .info
            .capabilities
            .contains(&"block_production".to_string()));
    }
}
