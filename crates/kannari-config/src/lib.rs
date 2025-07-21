// Copyright (c) KanariNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::proposer_config::ProposerConfig;
use crate::store_config::StoreConfig;
use anyhow::Result;
use clap::Parser;
use moveos_config::{temp_dir, DataDirPath};
use once_cell::sync::Lazy;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::genesis_config::GenesisConfig;
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID, RoochNetwork};
use rooch_types::service_status::ServiceStatus;
use rooch_types::service_type::ServiceType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt::Debug, path::Path, path::PathBuf};

pub mod config;
pub mod proposer_config;
pub mod server_config;
pub mod settings;
pub mod store_config;

pub const KANARI_DIR: &str = ".kanari";
pub const KANARI_CONFIR_DIR: &str = "kanari_config";
pub const KANARI_CLIENT_CONFIG: &str = "kanari.yaml";
pub const KANARI_KEYSTORE_FILENAME: &str = "kanari.keystore";


pub static R_DEFAULT_BASE_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs_next::home_dir()
        .expect("read home dir should ok")
        .join(".kanari")
});

#[derive(Debug, Clone, PartialEq)]
pub enum MapConfigValueSource {
    MapConfig,   // Value came from the presence of a key in the map configuration
    Environment, // Value came from the environment
    Default,     // Value came from a defined default value
    None,        // Value is not present in the map configuration, environment, or default value
}
