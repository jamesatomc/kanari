// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
pub struct KanariBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}

impl KanariBehaviour {
    pub fn new(local_peer_id: PeerId) -> Result<Self, Box<dyn std::error::Error>> {
        // Gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .max_transmit_size(262144)
            .build()
            .expect("Valid gossipsub config");

        // Create a gossipsub instance
        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_peer_id.into()),
            gossipsub_config,
        )?;

        // Subscribe to default topics
        let topics = vec![
            "kanari/blocks",
            "kanari/transactions",
            "kanari/consensus",
            "kanari/node-discovery",
        ];

        for topic_str in topics {
            let topic = gossipsub::IdentTopic::new(topic_str);
            gossipsub.subscribe(&topic)?;
            tracing::info!("Subscribed to topic: {}", topic_str);
        }

        // mDNS configuration
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Kademlia configuration
        let kademlia_config = kad::Config::default();
        let store = kad::store::MemoryStore::new(local_peer_id);
        let mut kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);

        // Set Kademlia mode to server (can respond to queries)
        kademlia.set_mode(Some(kad::Mode::Server));

        // Identify configuration
        let identify_config =
            identify::Config::new("/kanari/1.0.0".to_string(), local_peer_id.into())
                .with_interval(Duration::from_secs(60));
        let identify = identify::Behaviour::new(identify_config);

        // Ping configuration
        let ping_config = ping::Config::new()
            .with_interval(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(10));
        let ping = ping::Behaviour::new(ping_config);

        Ok(Self {
            gossipsub,
            mdns,
            kademlia,
            identify,
            ping,
        })
    }

    /// Publish a message to a gossipsub topic
    pub fn publish_message(
        &mut self,
        topic: &str,
        data: Vec<u8>,
    ) -> Result<(), gossipsub::PublishError> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.gossipsub.publish(topic, data)
    }

    /// Add a peer to Kademlia routing table
    pub fn add_address(&mut self, peer: PeerId, address: libp2p::Multiaddr) {
        self.kademlia.add_address(&peer, address);
    }

    /// Start bootstrap process
    pub fn bootstrap(&mut self) -> Result<kad::QueryId, kad::NoKnownPeers> {
        self.kademlia.bootstrap()
    }

    /// Get connected peers count
    pub fn connected_peers(&self) -> usize {
        self.gossipsub.all_peers().count()
    }
}

/// Events that can be emitted by the Kanari behaviour
#[derive(Debug)]
pub enum KanariEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(kad::Event),
    Identify(identify::Event),
    Ping(ping::Event),
}
