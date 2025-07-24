// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod behavior;
pub mod config;
pub mod message;
pub mod network;
pub mod node;
pub mod peer;
pub mod protocol;

pub use behavior::KanariBehaviour;
pub use config::P2PConfig;
pub use message::{Message, MessageType};
pub use network::P2PNetwork;
pub use node::{Node, NodeId, NodeInfo};
pub use peer::{Peer, PeerInfo, PeerManager};
pub use protocol::{Protocol, ProtocolEvent};

use anyhow::Result;

/// Re-export important types
pub use libp2p::{multiaddr::Multiaddr, PeerId};

/// Initialize the P2P networking module
pub fn init_p2p() -> Result<()> {
    tracing::info!("Initializing Kanari P2P networking module");
    Ok(())
}
