// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Local peer ID
    pub local_peer_id: Option<String>,

    /// Listening addresses
    pub listen_addresses: Vec<Multiaddr>,

    /// Bootstrap peers
    pub bootstrap_peers: Vec<Multiaddr>,

    /// Maximum number of connections
    pub max_connections: u32,

    /// Connection keep-alive timeout
    pub keep_alive_timeout: Duration,

    /// Connection idle timeout
    pub idle_connection_timeout: Duration,

    /// Enable mDNS discovery
    pub enable_mdns: bool,

    /// Enable Kademlia DHT
    pub enable_kademlia: bool,

    /// Gossipsub configuration
    pub gossipsub_config: GossipsubConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipsubConfig {
    /// Maximum message size
    pub max_message_size: usize,

    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Message validation mode
    pub validation_mode: String,

    /// Topics to subscribe to on startup
    pub default_topics: Vec<String>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            local_peer_id: None,
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/6778".parse().unwrap()],
            bootstrap_peers: vec![],
            max_connections: 50,
            keep_alive_timeout: Duration::from_secs(30),
            idle_connection_timeout: Duration::from_secs(60),
            enable_mdns: true,
            enable_kademlia: true,
            gossipsub_config: GossipsubConfig::default(),
        }
    }
}

impl Default for GossipsubConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            heartbeat_interval: Duration::from_secs(1),
            validation_mode: "strict".to_string(),
            default_topics: vec![
                "kanari/blocks".to_string(),
                "kanari/transactions".to_string(),
                "kanari/consensus".to_string(),
            ],
        }
    }
}

impl P2PConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_listen_addresses(mut self, addresses: Vec<Multiaddr>) -> Self {
        self.listen_addresses = addresses;
        self
    }

    pub fn with_bootstrap_peers(mut self, peers: Vec<Multiaddr>) -> Self {
        self.bootstrap_peers = peers;
        self
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.listen_addresses.is_empty() {
            anyhow::bail!("At least one listen address must be specified");
        }

        if self.max_connections == 0 {
            anyhow::bail!("max_connections must be greater than 0");
        }

        Ok(())
    }
}
