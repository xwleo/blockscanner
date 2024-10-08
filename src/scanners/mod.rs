pub mod bsc;
pub mod tron;

use crate::utils::error::AppError;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait BlockchainScanner: Send + Sync {
    fn chain_name(&self) -> &str;
    async fn get_latest_block(&self) -> Result<u64, AppError>;
    async fn scan_block(&self, block_num: u64) -> Result<Vec<Value>, AppError>;
    async fn filter_transactions(&self, transactions: Vec<Value>) -> Vec<Value>;
    fn get_start_block(&self) -> u64;

    async fn parse_transaction(
        &self,
        transaction: &Value,
        receipt: &Value,
    ) -> Result<Value, AppError>;
}
