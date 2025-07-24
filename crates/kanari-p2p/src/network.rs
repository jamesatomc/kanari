// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::behavior::KanariBehaviour;
use crate::config::P2PConfig;
use crate::message::{Message, MessageType, NodeInfoPayload};
use crate::node::{Node, NodeId, NodeInfo};
use crate::peer::{Peer, PeerManager, PeerStatus};

use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping, tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// P2P Network manager
pub struct P2PNetwork {
    swarm: Swarm<KanariBehaviour>,
    peer_manager: PeerManager,
    local_node: Node,
    config: P2PConfig,
    event_sender: Option<mpsc::UnboundedSender<NetworkEvent>>,
}

impl P2PNetwork {
    /// Create a new P2P network
    pub async fn new(config: P2PConfig, node: Node) -> Result<Self> {
        // Generate or use existing peer ID
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create behaviour
        let behaviour = KanariBehaviour::new(local_peer_id)?;

        // Create swarm
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        // Listen on configured addresses
        for addr in &config.listen_addresses {
            swarm.listen_on(addr.clone())?;
            info!("Listening on: {}", addr);
        }

        // Create peer manager
        let peer_manager =
            PeerManager::new(config.max_connections as usize, config.connection_timeout);

        Ok(Self {
            swarm,
            peer_manager,
            local_node: node,
            config,
            event_sender: None,
        })
    }

    /// Start the P2P network
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting P2P network for node: {}",
            self.local_node.info.name
        );

        // Start the local node
        self.local_node.start()?;

        // Connect to bootstrap peers
        self.connect_to_bootstrap_peers().await?;

        // Start bootstrap process
        if self.config.enable_kademlia {
            if let Err(e) = self.swarm.behaviour_mut().bootstrap() {
                warn!("Failed to start bootstrap: {:?}", e);
            }
        }

        Ok(())
    }

    /// Run the network event loop
    pub async fn run(&mut self) -> Result<()> {
        let mut cleanup_interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }
                _ = cleanup_interval.tick() => {
                    self.peer_manager.cleanup_stale_connections();
                }
            }
        }
    }

    /// Send a message to all connected peers
    pub fn broadcast_message(&mut self, message: Message) -> Result<()> {
        let topic = self.get_topic_for_message(&message.msg_type);
        let data = message.to_bytes()?;

        if let Err(e) = self.swarm.behaviour_mut().publish_message(&topic, data) {
            error!("Failed to broadcast message: {}", e);
            return Err(anyhow::anyhow!("Failed to broadcast message: {}", e));
        }

        info!("Broadcasted message type: {:?}", message.msg_type);
        Ok(())
    }

    /// Send a direct message to a specific peer
    pub fn send_direct_message(&mut self, peer_id: &PeerId, message: Message) -> Result<()> {
        // For now, we'll use gossipsub even for direct messages
        // In the future, we could implement a request-response protocol
        let topic = format!("kanari/direct/{}", peer_id);
        let data = message.to_bytes()?;

        if let Err(e) = self.swarm.behaviour_mut().publish_message(&topic, data) {
            error!("Failed to send direct message: {}", e);
            return Err(anyhow::anyhow!("Failed to send direct message: {}", e));
        }

        info!("Sent direct message to peer: {}", peer_id);
        Ok(())
    }

    /// Get network statistics
    pub fn get_stats(&self) -> NetworkStats {
        let peer_stats = self.peer_manager.get_stats();
        let node_stats = self.local_node.get_stats();

        NetworkStats {
            local_peer_id: format!("{}", self.swarm.local_peer_id()),
            connected_peers: self.swarm.behaviour().connected_peers(),
            node_stats,
            peer_stats,
            uptime_seconds: node_stats.uptime_seconds,
        }
    }

    /// Set event sender for external event handling
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<NetworkEvent>) {
        self.event_sender = Some(sender);
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        &mut self,
        event: libp2p::swarm::SwarmEvent<libp2p::swarm::behaviour::toggle::Toggle<KanariBehaviour>>,
    ) -> Result<()> {
        match event {
            libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
                // Handle behaviour-specific events
                // Note: This is a simplified approach. In a real implementation,
                // you'd need to properly handle the nested event types
                info!("Received behaviour event");
            }
            libp2p::swarm::SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!("Connection established with peer: {}", peer_id);

                // Add peer to peer manager
                let peer = Peer::new(
                    peer_id.to_string(),
                    endpoint.get_remote_address().to_string(),
                );
                if let Err(e) = self.peer_manager.add_peer(peer) {
                    warn!("Failed to add peer to manager: {}", e);
                } else {
                    self.peer_manager
                        .update_peer_status(&peer_id.to_string(), PeerStatus::Connected);
                }

                // Send event if handler is set
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(NetworkEvent::PeerConnected(peer_id.to_string()));
                }
            }
            libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!(
                    "Connection closed with peer: {} (cause: {:?})",
                    peer_id, cause
                );

                self.peer_manager
                    .update_peer_status(&peer_id.to_string(), PeerStatus::Disconnected);

                // Send event if handler is set
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(NetworkEvent::PeerDisconnected(peer_id.to_string()));
                }
            }
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }
            libp2p::swarm::SwarmEvent::IncomingConnection { .. } => {
                info!("Incoming connection");
            }
            libp2p::swarm::SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    warn!("Outgoing connection error to peer {}: {}", peer_id, error);
                    self.peer_manager
                        .update_peer_status(&peer_id.to_string(), PeerStatus::Failed);
                } else {
                    warn!("Outgoing connection error: {}", error);
                }
            }
            _ => {
                // Handle other events as needed
            }
        }
        Ok(())
    }

    /// Connect to bootstrap peers
    async fn connect_to_bootstrap_peers(&mut self) -> Result<()> {
        for addr in &self.config.bootstrap_peers.clone() {
            match self.swarm.dial(addr.clone()) {
                Ok(_) => {
                    info!("Dialing bootstrap peer: {}", addr);
                }
                Err(e) => {
                    warn!("Failed to dial bootstrap peer {}: {}", addr, e);
                }
            }
        }
        Ok(())
    }

    /// Get appropriate topic for message type
    fn get_topic_for_message(&self, msg_type: &MessageType) -> String {
        match msg_type {
            MessageType::BlockProposal
            | MessageType::BlockCommit
            | MessageType::BlockRequest
            | MessageType::BlockResponse => "kanari/blocks".to_string(),

            MessageType::TransactionBroadcast
            | MessageType::TransactionRequest
            | MessageType::TransactionResponse => "kanari/transactions".to_string(),

            MessageType::ConsensusProposal
            | MessageType::ConsensusVote
            | MessageType::ConsensusCommit => "kanari/consensus".to_string(),

            MessageType::NodeJoin
            | MessageType::NodeLeave
            | MessageType::NodeHeartbeat
            | MessageType::NodeInfo
            | MessageType::PeerDiscovery => "kanari/node-discovery".to_string(),

            MessageType::PeerConnection | MessageType::PeerDisconnection => {
                "kanari/peers".to_string()
            }

            MessageType::Custom(topic) => format!("kanari/custom/{}", topic),
        }
    }
}

/// Network events that can be sent to external handlers
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    MessageReceived(Message),
    BlockReceived(String),
    TransactionReceived(String),
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub local_peer_id: String,
    pub connected_peers: usize,
    pub node_stats: crate::node::NodeStats,
    pub peer_stats: crate::peer::PeerManagerStats,
    pub uptime_seconds: u64,
}
