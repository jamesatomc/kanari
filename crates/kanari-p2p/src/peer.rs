// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::node::{NodeId, NodeInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Peer connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PeerStatus {
    Connected,
    Connecting,
    Disconnected,
    Failed,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: NodeId,
    pub address: String,
    pub status: PeerStatus,
    pub last_seen: SystemTime,
    pub connection_time: Option<SystemTime>,
    pub version: String,
    pub capabilities: Vec<String>,
    pub latency: Option<Duration>,
    pub reputation_score: i32,
}

impl PeerInfo {
    pub fn new(id: NodeId, address: String) -> Self {
        Self {
            id,
            address,
            status: PeerStatus::Disconnected,
            last_seen: SystemTime::now(),
            connection_time: None,
            version: "unknown".to_string(),
            capabilities: vec![],
            latency: None,
            reputation_score: 0,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.status == PeerStatus::Connected
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }

    pub fn set_connected(&mut self) {
        self.status = PeerStatus::Connected;
        self.connection_time = Some(SystemTime::now());
        self.update_last_seen();
    }

    pub fn set_disconnected(&mut self) {
        self.status = PeerStatus::Disconnected;
        self.connection_time = None;
    }
}

/// Individual peer structure
#[derive(Debug, Clone)]
pub struct Peer {
    pub info: PeerInfo,
    pub node_info: Option<NodeInfo>,
}

impl Peer {
    pub fn new(id: NodeId, address: String) -> Self {
        Self {
            info: PeerInfo::new(id, address),
            node_info: None,
        }
    }

    pub fn with_node_info(mut self, node_info: NodeInfo) -> Self {
        self.node_info = Some(node_info);
        self
    }

    pub fn update_info(&mut self, node_info: NodeInfo) {
        self.info.version = node_info.version.clone();
        self.info.capabilities = node_info.capabilities.clone();
        self.node_info = Some(node_info);
        self.info.update_last_seen();
    }
}

/// Peer manager for handling all peer connections
#[derive(Debug)]
pub struct PeerManager {
    peers: HashMap<NodeId, Peer>,
    max_peers: usize,
    connection_timeout: Duration,
}

impl PeerManager {
    pub fn new(max_peers: usize, connection_timeout: Duration) -> Self {
        Self {
            peers: HashMap::new(),
            max_peers,
            connection_timeout,
        }
    }

    /// Add a new peer
    pub fn add_peer(&mut self, peer: Peer) -> anyhow::Result<()> {
        if self.peers.len() >= self.max_peers {
            // Find and remove least recently seen disconnected peer
            if let Some(peer_to_remove) = self.find_peer_to_remove() {
                self.peers.remove(&peer_to_remove);
                tracing::info!("Removed peer {} to make room for new peer", peer_to_remove);
            } else {
                anyhow::bail!(
                    "Cannot add peer: maximum peer limit reached and no removable peers found"
                );
            }
        }

        let peer_id = peer.info.id.clone();
        self.peers.insert(peer_id.clone(), peer);
        tracing::info!("Added peer: {}", peer_id);
        Ok(())
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &NodeId) -> Option<Peer> {
        if let Some(peer) = self.peers.remove(peer_id) {
            tracing::info!("Removed peer: {}", peer_id);
            Some(peer)
        } else {
            tracing::warn!("Attempted to remove non-existent peer: {}", peer_id);
            None
        }
    }

    /// Get a peer by ID
    pub fn get_peer(&self, peer_id: &NodeId) -> Option<&Peer> {
        self.peers.get(peer_id)
    }

    /// Get mutable reference to a peer
    pub fn get_peer_mut(&mut self, peer_id: &NodeId) -> Option<&mut Peer> {
        self.peers.get_mut(peer_id)
    }

    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<&Peer> {
        self.peers
            .values()
            .filter(|peer| peer.info.is_connected())
            .collect()
    }

    /// Get all peers
    pub fn get_all_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    /// Update peer status
    pub fn update_peer_status(&mut self, peer_id: &NodeId, status: PeerStatus) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            match status {
                PeerStatus::Connected => peer.info.set_connected(),
                PeerStatus::Disconnected => peer.info.set_disconnected(),
                _ => peer.info.status = status,
            }
            tracing::debug!("Updated peer {} status to {:?}", peer_id, status);
        }
    }

    /// Update peer information
    pub fn update_peer_info(&mut self, peer_id: &NodeId, node_info: NodeInfo) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.update_info(node_info);
            tracing::debug!("Updated info for peer: {}", peer_id);
        }
    }

    /// Check for timed out connections
    pub fn cleanup_stale_connections(&mut self) {
        let current_time = SystemTime::now();
        let timeout = self.connection_timeout;

        let stale_peers: Vec<NodeId> = self
            .peers
            .iter()
            .filter(|(_, peer)| {
                if let Ok(duration) = current_time.duration_since(peer.info.last_seen) {
                    duration > timeout && peer.info.status != PeerStatus::Connected
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        for peer_id in stale_peers {
            self.remove_peer(&peer_id);
            tracing::info!("Removed stale peer: {}", peer_id);
        }
    }

    /// Get peer statistics
    pub fn get_stats(&self) -> PeerManagerStats {
        let connected_count = self.get_connected_peers().len();
        let total_count = self.peers.len();

        PeerManagerStats {
            total_peers: total_count,
            connected_peers: connected_count,
            disconnected_peers: total_count - connected_count,
            max_peers: self.max_peers,
            average_reputation: self.calculate_average_reputation(),
        }
    }

    /// Find peer to remove when at capacity
    fn find_peer_to_remove(&self) -> Option<NodeId> {
        self.peers
            .iter()
            .filter(|(_, peer)| !peer.info.is_connected())
            .min_by_key(|(_, peer)| peer.info.last_seen)
            .map(|(id, _)| id.clone())
    }

    /// Calculate average reputation score
    fn calculate_average_reputation(&self) -> f64 {
        if self.peers.is_empty() {
            return 0.0;
        }

        let total: i32 = self
            .peers
            .values()
            .map(|peer| peer.info.reputation_score)
            .sum();
        total as f64 / self.peers.len() as f64
    }

    /// Get peers by capability
    pub fn get_peers_with_capability(&self, capability: &str) -> Vec<&Peer> {
        self.peers
            .values()
            .filter(|peer| {
                peer.info.capabilities.contains(&capability.to_string()) && peer.info.is_connected()
            })
            .collect()
    }

    /// Update peer latency
    pub fn update_peer_latency(&mut self, peer_id: &NodeId, latency: Duration) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.info.latency = Some(latency);
            peer.info.update_last_seen();
        }
    }

    /// Increase peer reputation
    pub fn increase_reputation(&mut self, peer_id: &NodeId, amount: i32) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.info.reputation_score += amount;
            tracing::debug!("Increased reputation for peer {} by {}", peer_id, amount);
        }
    }

    /// Decrease peer reputation
    pub fn decrease_reputation(&mut self, peer_id: &NodeId, amount: i32) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.info.reputation_score -= amount;
            tracing::debug!("Decreased reputation for peer {} by {}", peer_id, amount);
        }
    }
}

/// Peer manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerManagerStats {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub disconnected_peers: usize,
    pub max_peers: usize,
    pub average_reputation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info_creation() {
        let peer_info = PeerInfo::new("test-peer".to_string(), "127.0.0.1:8080".to_string());
        assert_eq!(peer_info.id, "test-peer");
        assert_eq!(peer_info.address, "127.0.0.1:8080");
        assert_eq!(peer_info.status, PeerStatus::Disconnected);
        assert!(!peer_info.is_connected());
    }

    #[test]
    fn test_peer_manager() {
        let mut manager = PeerManager::new(10, Duration::from_secs(30));

        let peer = Peer::new("test-peer".to_string(), "127.0.0.1:8080".to_string());
        manager.add_peer(peer).unwrap();

        assert_eq!(manager.get_all_peers().len(), 1);
        assert!(manager.get_peer(&"test-peer".to_string()).is_some());

        manager.update_peer_status(&"test-peer".to_string(), PeerStatus::Connected);
        assert_eq!(manager.get_connected_peers().len(), 1);
    }
}
