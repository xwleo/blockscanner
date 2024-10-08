mod scanners;
mod utils;

use crate::utils::{config::AppConfig, log::Logger};
use scanners::{bsc::BscScanner, tron::TronScanner, BlockchainScanner};
use std::error::Error;
use tokio::sync::mpsc;
use tracing::{error, info, instrument};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::new()?;

    println!("Log config: {:?}", config.log);

    Logger::init(&config.log)?;

    info!("Logger initialized");
    info!("Starting blockchain scanner...");

    let mut scanners: Vec<Box<dyn BlockchainScanner>> = Vec::new();

    if config.tron.enable {
        let tron_scanner = TronScanner::new(&config.tron.api_url, config.tron.start_block)?;
        scanners.push(Box::new(tron_scanner));
    }

    if config.bsc.enable {
        let bsc_scanner = BscScanner::new(&config.bsc.api_url, config.bsc.start_block)?;
        scanners.push(Box::new(bsc_scanner));
    }

    let (tx, mut rx) = mpsc::channel(100);

    for scanner in scanners {
        let tx = tx.clone();
        let chain_name = scanner.chain_name().to_string();
        tokio::spawn(async move {
            let mut current_block = scanner.get_start_block();
            loop {
                if let Err(e) = scan_block(&*scanner, current_block, &chain_name).await {
                    error!("Error scanning {} block: {:?}", chain_name, e);
                }
                if tx.send(()).await.is_err() {
                    break;
                }
                current_block += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            }
        });
    }

    while rx.recv().await.is_some() {}

    Ok(())
}

#[instrument(skip(scanner))]
async fn scan_block(
    scanner: &dyn BlockchainScanner,
    block_num: u64,
    chain_name: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("Scanning {} block: {}", chain_name, block_num);

    match scanner.scan_block(block_num).await {
        Ok(transactions) => {
            let filtered_transactions = scanner.filter_transactions(transactions).await;
            if !filtered_transactions.is_empty() {
                info!(
                    "Relevant transactions for {} block {}:",
                    chain_name, block_num
                );
                for tx in filtered_transactions {
                    println!("{}", serde_json::to_string_pretty(&tx).unwrap());
                }
            }
        }
        Err(e) => {
            error!("Error scanning {} block {}: {:?}", chain_name, block_num, e);
        }
    }

    Ok(())
}
