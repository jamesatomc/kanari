// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// Block-related messages
    BlockProposal,
    BlockCommit,
    BlockRequest,
    BlockResponse,

    /// Transaction-related messages
    TransactionBroadcast,
    TransactionRequest,
    TransactionResponse,

    /// Node management messages
    NodeJoin,
    NodeLeave,
    NodeHeartbeat,
    NodeInfo,

    /// Consensus messages
    ConsensusProposal,
    ConsensusVote,
    ConsensusCommit,

    /// Network management
    PeerDiscovery,
    PeerConnection,
    PeerDisconnection,

    /// Custom/Future extension
    Custom(String),
}

/// P2P Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: Uuid,

    /// Message type
    pub msg_type: MessageType,

    /// Message payload (serialized data)
    pub payload: Vec<u8>,

    /// Sender peer ID
    pub sender: Option<String>,

    /// Target peer ID (None for broadcast)
    pub target: Option<String>,

    /// Timestamp when message was created
    pub timestamp: u64,

    /// Message metadata
    pub metadata: HashMap<String, String>,

    /// TTL (Time To Live) for message propagation
    pub ttl: u32,
}

impl Message {
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type,
            payload,
            sender: None,
            target: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
            ttl: 10, // Default TTL
        }
    }

    pub fn with_sender(mut self, sender: String) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = ttl;
        self
    }

    /// Check if message has expired based on TTL
    pub fn is_expired(&self) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        current_time - self.timestamp > (self.ttl as u64)
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

/// Block proposal message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposalPayload {
    pub block_number: u128,
    pub block_hash: String,
    pub parent_hash: String,
    pub proposer: String,
    pub timestamp: u64,
    pub transactions: Vec<String>, // Transaction hashes
}

/// Transaction broadcast payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub tx_hash: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: String,
}

/// Node information payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoPayload {
    pub node_id: String,
    pub node_type: String,
    pub version: String,
    pub chain_id: u64,
    pub listening_addresses: Vec<String>,
    pub capabilities: Vec<String>,
    pub initial_balance: u64, // Add initial balance field
}

/// Consensus vote payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVotePayload {
    pub block_hash: String,
    pub block_number: u128,
    pub voter_id: String,
    pub vote_type: VoteType,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    Approve,
    Reject,
    Abstain,
}
