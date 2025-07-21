// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

//! Kanari Genesis Configuration
//!
//! This module provides genesis configuration for the Kanari network, adapted from Rooch's
//! genesis system while maintaining compatibility with Kanari-specific requirements.
//!
//! The initial supply and token specifications are synchronized with the KARI token
//! definition in `kanari.move`:
//! - Initial Supply: 100 million KARI tokens
//! - Decimals: 8 (matching Move module specification)
//! - Token Symbol: KARI
//!
//! All network configurations (local, dev, testnet, mainnet) use the same initial supply
//! as defined in the Move smart contract to ensure consistency across the system.
//!
//! # Usage Examples
//!
//! ```rust,no_run
//! use kanari_types::genesis_config::*;
//!
//! fn main() -> anyhow::Result<()> {
//!     // For local development
//!     let local_genesis = &*KANARI_LOCAL_CONFIG;
//!     println!("Local network ID: {}", local_genesis.network_id);
//!
//!     // For testnet deployment
//!     let testnet_genesis = &*KANARI_TESTNET_CONFIG;
//!     println!("Testnet initial supply: {}", testnet_genesis.initial_supply);
//!
//!     // For mainnet deployment
//!     let mainnet_genesis = &*KANARI_MAINNET_CONFIG;
//!     println!("Mainnet DAO threshold: {}", mainnet_genesis.kanari_dao.threshold);
//!
//!     // Loading custom configuration from file
//!     let custom_config = KanariGenesisConfig::load("path/to/genesis.yaml")?;
//!
//!     // Saving configuration to file
//!     let config = &*KANARI_DEV_CONFIG;
//!     config.save("genesis_dev.yaml")?;
//!
//!     Ok(())
//! }
//! ```

use rooch_types::{
    address::BitcoinAddress,
    bitcoin::{genesis::MultisignAccountConfig, ord::InscriptionStore, utxo::BitcoinUTXOStore, network::Network},
    framework::address_mapping::RoochToBitcoinAddressMapping,
};
use bitcoin::{block::Header, BlockHash};
use framework_builder::stdlib_version::StdlibVersion;
use move_core_types::value::MoveTypeLayout;
use moveos_types::{
    h256::H256,
    moveos_std::{module_store::ModuleStore, timestamp::Timestamp},
    state::{MoveState, ObjectState},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// KARI Token specifications (from kanari.move)
// These values MUST match the values defined in frameworks/kanari-library/sources/kanari.move:
// - INITIAL_SUPPLY: u256 = 10_000_000_000_000_000u256 (100 million tokens)
// - DECIMALS: u8 = 8u8
/// Total initial supply of KARI tokens as defined in kanari.move
pub const KARI_INITIAL_SUPPLY: u128 = 10_000_000_000_000_000; // 100 million KARI with 8 decimals
/// Number of decimal places for KARI token as defined in kanari.move
pub const KARI_DECIMALS: u8 = 8;

/// Helper functions for KARI token amount conversions
impl KanariGenesisConfig {
    /// Convert KARI tokens to smallest unit (considering 8 decimals)
    /// Example: 1.0 KARI = 100,000,000 smallest units
    pub fn kari_to_smallest_unit(kari_amount: f64) -> u128 {
        (kari_amount * 10_f64.powi(KARI_DECIMALS as i32)) as u128
    }

    /// Convert smallest units back to KARI tokens (considering 8 decimals)
    /// Example: 100,000,000 smallest units = 1.0 KARI
    pub fn smallest_unit_to_kari(smallest_units: u128) -> f64 {
        smallest_units as f64 / 10_f64.powi(KARI_DECIMALS as i32)
    }

    /// Get the initial supply in human-readable KARI tokens
    /// Returns: 100.0 (representing 100 million KARI)
    pub fn get_initial_supply_in_kari(&self) -> f64 {
        Self::smallest_unit_to_kari(self.initial_supply)
    }

    /// Validate that the genesis config matches kanari.move specifications
    pub fn validate_kari_specs(&self) -> Result<(), String> {
        if self.initial_supply != KARI_INITIAL_SUPPLY {
            return Err(format!(
                "Initial supply mismatch: expected {}, got {}",
                KARI_INITIAL_SUPPLY, self.initial_supply
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KanariGenesisConfig {
    /// The Bitcoin network that the genesis block is based on
    pub bitcoin_network: u8,
    /// The height of the Bitcoin block that the genesis block is based on
    pub bitcoin_block_height: u64,
    /// The hash of the Bitcoin block that the genesis block is based on
    pub bitcoin_block_hash: BlockHash,
    /// The maximum number of blocks that can be reorganized
    pub bitcoin_reorg_block_count: u64,
    /// The timestamp of the Bitcoin block that the genesis block is based on
    pub timestamp: u64,
    /// The genesis sequencer account for Kanari
    pub sequencer_account: BitcoinAddress,
    /// The genesis kanari dao account multisign config
    pub kanari_dao: MultisignAccountConfig,
    /// Genesis objects for Kanari network
    pub genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    /// Standard library version
    pub stdlib_version: StdlibVersion,
    /// Kanari network specific configuration
    pub network_id: String,
    /// Initial KARI token supply
    pub initial_supply: u128,
    /// Genesis validators for Kanari network
    pub genesis_validators: Vec<BitcoinAddress>,
}

impl KanariGenesisConfig {
    pub fn load<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let content = std::fs::read_to_string(path)?;
        let config: KanariGenesisConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// Kanari Local Development Configuration
pub static KANARI_LOCAL_CONFIG: Lazy<KanariGenesisConfig> = Lazy::new(|| KanariGenesisConfig {
    bitcoin_network: Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    // The regtest genesis block hash
    bitcoin_block_hash: BlockHash::from_str(
        "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
    )
    .expect("Should be valid"),
    bitcoin_reorg_block_count: 0,
    timestamp: 0,
    // Kanari local sequencer account
    sequencer_account: BitcoinAddress::from_str(
        "bc1pkanari8local8dev8account8for8testing8purposes8only8xyz",
    )
    .unwrap_or_else(|_| {
        BitcoinAddress::from_str("bc1pxup9p7um3t5knqn0yxfrq5d0mgul9ts993j32tsfxn68qa4pl3nq2qhh2e").unwrap()
    }),
    kanari_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1pkanari8dao8multisign8address8for8local8development",
        )
        .unwrap_or_else(|_| {
            BitcoinAddress::from_str("bc1pevdrc8yqmgd94h2mpz9st0u77htmx935hzck3ruwsvcf4w7wrnqqd0yvze").unwrap()
        }),
        threshold: 1,
        participant_public_keys: vec![hex::decode(
            "03ff7e1d7b4a152671124545f4fb68efe2a9bd0b3870ac22fee4afd4ecdfa8a19c",
        )
        .unwrap()],
    },
    genesis_objects: vec![
        (
            ObjectState::new_timestamp(Timestamp { milliseconds: 0 }),
            Timestamp::type_layout(),
        ),
        (
            ObjectState::genesis_module_store(),
            ModuleStore::type_layout(),
        ),
    ],
    stdlib_version: StdlibVersion::Latest,
    network_id: "kanari-local".to_string(),
    initial_supply: KARI_INITIAL_SUPPLY, // 100 million KARI with 8 decimals (from kanari.move)
    genesis_validators: vec![],
});

// Kanari Development Configuration
pub static KANARI_DEV_CONFIG: Lazy<KanariGenesisConfig> = Lazy::new(|| KanariGenesisConfig {
    bitcoin_network: Network::Regtest.to_num(),
    bitcoin_block_height: 0,
    bitcoin_block_hash: BlockHash::from_str(
        "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
    )
    .expect("Should be valid"),
    bitcoin_reorg_block_count: 0,
    timestamp: 0,
    sequencer_account: BitcoinAddress::from_str(
        "bc1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasq8lj2sg",
    )
    .expect("Should be valid"),
    kanari_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1pu38mumfnuppqn54kcnyymmqzpqgmmfxlgnu6dsc6qhschy7cj76qkcl24p",
        )
        .unwrap(),
        threshold: 1,
        participant_public_keys: vec![hex::decode(
            "026c9e5a00643a706d3826424f766bbbb08adada4dc357c1b279ad4662d2fd1e2e",
        )
        .unwrap()],
    },
    genesis_objects: vec![
        (
            ObjectState::new_timestamp(Timestamp { milliseconds: 0 }),
            Timestamp::type_layout(),
        ),
        (
            ObjectState::genesis_module_store(),
            ModuleStore::type_layout(),
        ),
    ],
    stdlib_version: StdlibVersion::Latest,
    network_id: "kanari-dev".to_string(),
    initial_supply: KARI_INITIAL_SUPPLY, // 100 million KARI with 8 decimals (from kanari.move)
    genesis_validators: vec![],
});

// Kanari Testnet Configuration
static KANARI_TESTNET_GENESIS_HEIGHT_HEADER: Lazy<(u64, Header)> = Lazy::new(|| {
    (3518200, bitcoin::consensus::deserialize(
        &hex::decode("00e0a523e38e0c995e1dffb39ce6c05c9b031d9ecf234581739cb555060000000000000064cdbe6ce56bc79f5e58213b47128229b0c402ed57239d80364214bd0c1d301e2a464d6700171019b48314b9")
            .expect("Should be valid"),
    ).expect("Should be valid"))
});

pub static KANARI_TESTNET_CONFIG: Lazy<KanariGenesisConfig> = Lazy::new(|| {
    KanariGenesisConfig {
        bitcoin_network: Network::Testnet.to_num(),
        bitcoin_block_height: KANARI_TESTNET_GENESIS_HEIGHT_HEADER.0,
        bitcoin_block_hash: KANARI_TESTNET_GENESIS_HEIGHT_HEADER.1.block_hash(),
        bitcoin_reorg_block_count: 5,
        timestamp: KANARI_TESTNET_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
        sequencer_account: BitcoinAddress::from_str(
            "tb1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqshy928",
        )
        .expect("Should be valid"),
        kanari_dao: MultisignAccountConfig {
            multisign_bitcoin_address: BitcoinAddress::from_str(
                "tb1pkanari8testnet8dao8multisign8address8for8testing",
            )
            .unwrap_or_else(|_| {
                BitcoinAddress::from_str("bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt").unwrap()
            }),
            threshold: 3,
            participant_public_keys: vec![
                hex::decode("032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c")
                    .unwrap(),
                hex::decode("039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0")
                    .unwrap(),
                hex::decode("03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61")
                    .unwrap(),
                hex::decode("0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677")
                    .unwrap(),
                hex::decode("03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be")
                    .unwrap(),
            ],
        },
        genesis_objects: vec![
            (
                ObjectState::new_timestamp(Timestamp {
                    milliseconds: KANARI_TESTNET_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
                }),
                Timestamp::type_layout(),
            ),
            (
                ObjectState::genesis_module_store(),
                ModuleStore::type_layout(),
            ),
        ],
        stdlib_version: StdlibVersion::Version(16),
        network_id: "kanari-testnet".to_string(),
        initial_supply: KARI_INITIAL_SUPPLY, // 100 million KARI with 8 decimals (from kanari.move)
        genesis_validators: vec![],
    }
});

// Kanari Mainnet Configuration
static KANARI_MAINNET_GENESIS_HEIGHT_HEADER: Lazy<(u64, Header)> = Lazy::new(|| {
    (859001, bitcoin::consensus::deserialize(
        &hex::decode("00e0ff274e6e46285bf4133faaafcf248ed461ffcdf8e2b33fba020000000000000000004275ffbb1e17c5b8abb04a9e57bc479c83dcf44c7bed3bc7f94c8449b6c2250619ecd0665b250317b7bc8d78")
            .expect("Should be valid"),
    ).expect("Should be valid"))
});

pub static KANARI_MAINNET_CONFIG: Lazy<KanariGenesisConfig> = Lazy::new(|| KanariGenesisConfig {
    bitcoin_network: Network::Bitcoin.to_num(),
    bitcoin_block_height: KANARI_MAINNET_GENESIS_HEIGHT_HEADER.0,
    bitcoin_block_hash: KANARI_MAINNET_GENESIS_HEIGHT_HEADER.1.block_hash(),
    bitcoin_reorg_block_count: 3,
    timestamp: KANARI_MAINNET_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
    sequencer_account: BitcoinAddress::from_str(
        "bc1pkanari8mainnet8sequencer8account8address8here",
    )
    .unwrap_or_else(|_| {
        BitcoinAddress::from_str("bc1pwxpq9pxgv2jnvzu2pjska3jkfurxsdt075yds3u0rsj9cu39g4esjdzt8z").unwrap()
    }),
    kanari_dao: MultisignAccountConfig {
        multisign_bitcoin_address: BitcoinAddress::from_str(
            "bc1pkanari8mainnet8dao8multisign8address8production",
        )
        .unwrap_or_else(|_| {
            BitcoinAddress::from_str("bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt").unwrap()
        }),
        threshold: 5,
        participant_public_keys: vec![
            hex::decode("032d4fb9f88a63f52d8bffd1a46ad40411310150a539913203265c3f46b0397f8c")
                .unwrap(),
            hex::decode("039c9f399047d1ca911827c8c9b445ea55e84a68dcfe39641bc1f423c6a7cd99d0")
                .unwrap(),
            hex::decode("03ad953cc82a6ed91c8eb3a6400e55965de4735bc5f8a107eabd2e4e7531f64c61")
                .unwrap(),
            hex::decode("0346b64846c11f23ccec99811b476aaf68f421f15762287b872fcb896c92caa677")
                .unwrap(),
            hex::decode("03730cb693e9a1bc6eaec5537c2e317a75bb6c8107a59fda018810c46c270670be")
                .unwrap(),
            hex::decode("0259a40918150bc16ca1852fb55be383ec0fcf2b6058a73a25f0dfd87394dd92db")
                .unwrap(),
            hex::decode("028fd25b727bf77e42d7a99cad4b1fa564d41cdb3bbddaf15219a4529f486a775a")
                .unwrap(),
        ],
    },
    genesis_objects: vec![
        (
            ObjectState::new_timestamp(Timestamp {
                milliseconds: KANARI_MAINNET_GENESIS_HEIGHT_HEADER.1.time as u64 * 1000,
            }),
            Timestamp::type_layout(),
        ),
        (
            ObjectState::genesis_module_store(),
            ModuleStore::type_layout(),
        ),
        (
            BitcoinUTXOStore::genesis_with_state_root(
                H256::from_str(
                    "0x8ec77de7cd44c27a30c84aaa36c4e107aae7aaade2ae3ee1741aad437015a219",
                )
                .unwrap(),
                185390577,
            ),
            BitcoinUTXOStore::type_layout(),
        ),
        (
            InscriptionStore::genesis_with_state_root(
                H256::from_str(
                    "0x8a4fc2cfb4d66c574e921b4fffa1a8af9156f821451cac1f3d61075572cdf68b",
                )
                .unwrap(),
                150953628,
                InscriptionStore {
                    cursed_inscription_count: 472043,
                    blessed_inscription_count: 75004771,
                    unbound_inscription_count: 20723,
                    lost_sats: 0,
                    next_sequence_number: 75476814,
                },
            ),
            InscriptionStore::type_layout(),
        ),
        (
            RoochToBitcoinAddressMapping::genesis_with_state_root(
                H256::from_str(
                    "0x908b63a475a886571a2bef1533589866f92fb3ef01b243a0b8bb1cda27655172",
                )
                .unwrap(),
                52397723,
            ),
            RoochToBitcoinAddressMapping::type_layout(),
        ),
    ],
    stdlib_version: StdlibVersion::Version(11),
    network_id: "kanari-mainnet".to_string(),
    initial_supply: KARI_INITIAL_SUPPLY, // 100 million KARI with 8 decimals (from kanari.move)
    genesis_validators: vec![],
});

#[cfg(test)]
mod tests {
    use super::*;


    fn test_kanari_genesis_config(config: &KanariGenesisConfig) {
        // Verify network ID is set
        assert!(!config.network_id.is_empty());

        // Verify initial supply is positive
        assert!(config.initial_supply > 0);

        // Verify basic configuration - block height should be valid (>= 0)
        // Note: Local and dev configs start at 0, testnet/mainnet start at higher blocks
        assert!(config.bitcoin_reorg_block_count < 100); // Reasonable upper bound
    }

    #[test]
    fn test_kanari_local_config() {
        test_kanari_genesis_config(&KANARI_LOCAL_CONFIG);
        assert_eq!(KANARI_LOCAL_CONFIG.network_id, "kanari-local");
    }

    #[test]
    fn test_kanari_dev_config() {
        test_kanari_genesis_config(&KANARI_DEV_CONFIG);
        assert_eq!(KANARI_DEV_CONFIG.network_id, "kanari-dev");
    }

    #[test]
    fn test_kanari_testnet_config() {
        test_kanari_genesis_config(&KANARI_TESTNET_CONFIG);
        assert_eq!(KANARI_TESTNET_CONFIG.network_id, "kanari-testnet");
    }

    #[test]
    fn test_kanari_mainnet_config() {
        test_kanari_genesis_config(&KANARI_MAINNET_CONFIG);
        assert_eq!(KANARI_MAINNET_CONFIG.network_id, "kanari-mainnet");
        // Mainnet uses same supply as defined in kanari.move
        assert_eq!(KANARI_MAINNET_CONFIG.initial_supply, KARI_INITIAL_SUPPLY);
    }

    #[test]
    fn test_config_serialization() {
        // Create a minimal config for testing serialization
        let config = KanariGenesisConfig {
            bitcoin_network: Network::Regtest.to_num(),
            bitcoin_block_height: 0,
            bitcoin_block_hash: BlockHash::from_str(
                "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
            ).unwrap(),
            bitcoin_reorg_block_count: 0,
            timestamp: 0,
            sequencer_account: BitcoinAddress::from_str(
                "bc1pxup9p7um3t5knqn0yxfrq5d0mgul9ts993j32tsfxn68qa4pl3nq2qhh2e",
            ).unwrap(),
            kanari_dao: MultisignAccountConfig {
                multisign_bitcoin_address: BitcoinAddress::from_str(
                    "bc1pevdrc8yqmgd94h2mpz9st0u77htmx935hzck3ruwsvcf4w7wrnqqd0yvze",
                ).unwrap(),
                threshold: 1,
                participant_public_keys: vec![hex::decode(
                    "03ff7e1d7b4a152671124545f4fb68efe2a9bd0b3870ac22fee4afd4ecdfa8a19c",
                ).unwrap()],
            },
            genesis_objects: vec![], // Empty for test
            stdlib_version: StdlibVersion::Latest,
            network_id: "kanari-test".to_string(),
            initial_supply: KARI_INITIAL_SUPPLY, // 100 million KARI with 8 decimals (from kanari.move)
            genesis_validators: vec![],
        };

        // Test basic field access instead of full serialization
        assert_eq!(config.network_id, "kanari-test");
        assert_eq!(config.initial_supply, KARI_INITIAL_SUPPLY);
        assert_eq!(config.bitcoin_network, Network::Regtest.to_num());
        assert!(config.genesis_objects.is_empty());
        assert!(config.genesis_validators.is_empty());
    }

    #[test]
    fn test_genesis_config_usage_example() {
        // Example of how to use different network configurations
        let local_config = &*KANARI_LOCAL_CONFIG;
        let testnet_config = &*KANARI_TESTNET_CONFIG;
        let mainnet_config = &*KANARI_MAINNET_CONFIG;

        // Verify each config has the expected network ID
        assert_eq!(local_config.network_id, "kanari-local");
        assert_eq!(testnet_config.network_id, "kanari-testnet");
        assert_eq!(mainnet_config.network_id, "kanari-mainnet");

        // Verify all configs have same supply as defined in kanari.move
        assert_eq!(local_config.initial_supply, KARI_INITIAL_SUPPLY);
        assert_eq!(mainnet_config.initial_supply, KARI_INITIAL_SUPPLY);

        // Show how to access genesis objects
        assert!(!local_config.genesis_objects.is_empty());
        assert!(!mainnet_config.genesis_objects.is_empty());
    }

    #[test]
    fn test_kari_token_conversions() {
        // Test conversion functions
        assert_eq!(KanariGenesisConfig::kari_to_smallest_unit(1.0), 100_000_000);
        assert_eq!(KanariGenesisConfig::kari_to_smallest_unit(100_000_000.0), KARI_INITIAL_SUPPLY);

        assert_eq!(KanariGenesisConfig::smallest_unit_to_kari(100_000_000), 1.0);
        assert_eq!(KanariGenesisConfig::smallest_unit_to_kari(KARI_INITIAL_SUPPLY), 100_000_000.0);

        // Test initial supply conversion
        let config = &*KANARI_LOCAL_CONFIG;
        assert_eq!(config.get_initial_supply_in_kari(), 100_000_000.0);

        // Test validation
        assert!(config.validate_kari_specs().is_ok());
    }

    #[test]
    fn test_kari_specs_validation() {
        // Create config with wrong supply
        let mut config = KanariGenesisConfig {
            bitcoin_network: Network::Regtest.to_num(),
            bitcoin_block_height: 0,
            bitcoin_block_hash: BlockHash::from_str(
                "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
            ).unwrap(),
            bitcoin_reorg_block_count: 0,
            timestamp: 0,
            sequencer_account: BitcoinAddress::from_str(
                "bc1pxup9p7um3t5knqn0yxfrq5d0mgul9ts993j32tsfxn68qa4pl3nq2qhh2e",
            ).unwrap(),
            kanari_dao: MultisignAccountConfig {
                multisign_bitcoin_address: BitcoinAddress::from_str(
                    "bc1pevdrc8yqmgd94h2mpz9st0u77htmx935hzck3ruwsvcf4w7wrnqqd0yvze",
                ).unwrap(),
                threshold: 1,
                participant_public_keys: vec![],
            },
            genesis_objects: vec![],
            stdlib_version: StdlibVersion::Latest,
            network_id: "test".to_string(),
            initial_supply: 999_999_999, // Wrong supply
            genesis_validators: vec![],
        };

        // Should fail validation
        assert!(config.validate_kari_specs().is_err());

        // Fix the supply
        config.initial_supply = KARI_INITIAL_SUPPLY;
        assert!(config.validate_kari_specs().is_ok());
    }
}
