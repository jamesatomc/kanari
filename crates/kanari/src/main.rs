use anyhow::Result;
use clap::{Parser, Subcommand};
use kanari_config::KanariOpt;
use kanari_db::RoochDB;
use kanari_types::block::Block;
use moveos_types::h256::H256;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{error, info, warn};

#[derive(Parser)]
#[clap(name = "kari", author = "The Kanari Core Contributors")]
#[clap(about = "Kanari - A high-performance blockchain platform")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Kanari node
    Start {
        #[clap(flatten)]
        config: KanariOpt,
    },
    /// Generate a new wallet
    Keytool {
        #[clap(subcommand)]
        keytool_command: KeytoolCommands,
    },
}

#[derive(Subcommand)]
enum KeytoolCommands {
    /// Generate a new wallet
    Generate,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config } => {
            info!("Starting Kanari node...");
            start_node(config).await?;
        }
        Commands::Keytool { keytool_command } => match keytool_command {
            KeytoolCommands::Generate => {
                info!("Generating new wallet...");
                generate_wallet().await?;
            }
        },
    }

    Ok(())
}

async fn start_node(mut config: KanariOpt) -> Result<()> {
    // Initialize the configuration first
    config.init()?;
    
    info!("Kanari node configuration: {:?}", config);
    info!("Starting Kanari blockchain node...");

    // Initialize the database
    let registry = prometheus::Registry::new();
    let db = match RoochDB::init(&config.store, &registry) {
        Ok(db) => {
            info!("Database initialized successfully");
            db
        }
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(e);
        }
    };

    info!("Node is running on port: {:?}", config.port.unwrap_or(6767));
    info!(
        "Data directory: {:?}",
        config
            .base_data_dir
            .unwrap_or_else(|| std::path::PathBuf::from(".kanari"))
    );

    // Display information about existing blocks
    match db.get_latest_block_number() {
        Ok(Some(latest_block_number)) => {
            info!("Latest block in database: #{}", latest_block_number);
            // Display the latest block details
            if let Ok(Some(latest_block)) = db.get_block(latest_block_number) {
                info!("Latest block details: {:?}", latest_block);
            }
        }
        Ok(None) => {
            info!("No blocks found in database. Starting fresh blockchain.");
        }
        Err(e) => {
            warn!("Error checking for existing blocks: {}", e);
        }
    }

    // Start with the next block number
    let mut block_number = match db.get_latest_block_number()? {
        Some(latest) => latest + 1,
        None => 1,
    };

    // Create a sample block every 10 seconds to demonstrate block saving functionality
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        block_number += 1;
        match create_and_save_block(&db, block_number).await {
            Ok(block_hash) => {
                info!(
                    "Successfully created and saved block #{} with hash: {}",
                    block_number,
                    hex::encode(block_hash.as_bytes())
                );
            }
            Err(e) => {
                error!("Failed to create block #{}: {}", block_number, e);
            }
        }
    }
}

async fn generate_wallet() -> Result<()> {
    info!("Wallet generation functionality will be implemented here");
    // TODO: Implement wallet generation logic
    Ok(())
}

async fn create_and_save_block(db: &RoochDB, block_number: u128) -> Result<H256> {
    // Get current timestamp
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Create a new block
    let prev_hash = if block_number == 1 {
        H256::zero() // Genesis block
    } else {
        // In a real implementation, this would be the hash of the previous block
        // For now, let's try to get the previous block's hash
        match db.get_block(block_number - 1)? {
            Some(prev_block) => prev_block.batch_hash,
            None => H256::zero(), // Fallback if previous block not found
        }
    };

    let batch_hash = H256::random(); // In a real implementation, this would be computed from transactions
    let tx_accumulator_root = H256::random();
    let state_root = H256::random();

    let block = Block::new(
        block_number,
        0, // batch_size - no transactions in this demo
        batch_hash,
        prev_hash,
        tx_accumulator_root,
        state_root,
    );

    info!("Created block #{} at timestamp {}", block_number, timestamp);

    // Actually save the block to the database
    match db.save_block(&block) {
        Ok(()) => {
            info!("Block #{} successfully saved to database", block_number);
        }
        Err(e) => {
            error!("Failed to save block #{} to database: {}", block_number, e);
            return Err(e);
        }
    }

    // Return the batch_hash as the block identifier
    Ok(batch_hash)
}
