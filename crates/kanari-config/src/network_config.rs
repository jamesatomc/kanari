// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Default values for networking configuration
pub const DEFAULT_P2P_PORT: u16 = 6778;
pub const DEFAULT_MAX_PEERS: usize = 50;
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30; // seconds
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 60; // seconds
pub const DEFAULT_DISCOVERY_INTERVAL: u64 = 120; // seconds

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct NetworkConfig {
    /// The port for P2P networking
    #[clap(long, default_value_t = DEFAULT_P2P_PORT)]
    pub p2p_port: u16,

    /// Maximum number of peer connections
    #[clap(long, default_value_t = DEFAULT_MAX_PEERS)]
    pub max_peers: usize,

    /// Connection timeout in seconds
    #[clap(skip)]
    pub connection_timeout: Duration,

    /// Heartbeat interval in seconds
    #[clap(skip)]
    pub heartbeat_interval: Duration,

    /// Node discovery interval in seconds
    #[clap(skip)]
    pub discovery_interval: Duration,

    /// List of bootstrap nodes (address:port)
    #[clap(long, value_delimiter = ',')]
    pub bootstrap_nodes: Vec<String>,

    /// External address for this node (optional)
    #[clap(long)]
    pub external_address: Option<SocketAddr>,

    /// Enable node discovery
    #[clap(long, default_value_t = true)]
    pub enable_discovery: bool,

    /// Network identifier/chain ID
    #[clap(long, default_value_t = 3)]
    pub network_id: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            p2p_port: DEFAULT_P2P_PORT,
            max_peers: DEFAULT_MAX_PEERS,
            connection_timeout: Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT),
            heartbeat_interval: Duration::from_secs(DEFAULT_HEARTBEAT_INTERVAL),
            discovery_interval: Duration::from_secs(DEFAULT_DISCOVERY_INTERVAL),
            bootstrap_nodes: vec![],
            external_address: None,
            enable_discovery: true,
            network_id: 3, // Default to dev network
        }
    }
}

impl NetworkConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.p2p_port = port;
        self
    }

    pub fn with_bootstrap_nodes(mut self, nodes: Vec<String>) -> Self {
        self.bootstrap_nodes = nodes;
        self
    }

    pub fn with_max_peers(mut self, max_peers: usize) -> Self {
        self.max_peers = max_peers;
        self
    }

    pub fn with_network_id(mut self, network_id: u64) -> Self {
        self.network_id = network_id;
        self
    }

    /// Validate the network configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_peers == 0 {
            anyhow::bail!("max_peers must be greater than 0");
        }

        if self.p2p_port == 0 {
            anyhow::bail!("p2p_port must be greater than 0");
        }

        // Validate bootstrap nodes format
        for node in &self.bootstrap_nodes {
            if let Err(_) = node.parse::<SocketAddr>() {
                anyhow::bail!("Invalid bootstrap node address: {}", node);
            }
        }

        Ok(())
    }
}
