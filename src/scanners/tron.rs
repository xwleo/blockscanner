use crate::scanners::BlockchainScanner;
use crate::utils::error::AppError;
use crate::utils::tron::hex_to_tron_address;
use async_trait::async_trait;
use num_bigint::BigInt;
use num_traits::Num;
use reqwest::Client;
use serde_json::{json, Value};

pub struct TronScanner {
    api_url: String,
    client: Client,
    start_block: u64,
}

impl TronScanner {
    pub fn new(api_url: &str, start_block: u64) -> Result<Self, AppError> {
        let api_url = api_url.trim_end_matches('/').to_string();
        Ok(TronScanner {
            api_url,
            client: Client::new(),
            start_block,
        })
    }

    async fn make_http_request(&self, method: &str, params: Value) -> Result<Value, AppError> {
        let url = format!("{}/{}", self.api_url, method);
        let response = self
            .client
            .post(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(AppError::ApiError(format!(
                "HTTP error: {}, body: {}",
                status, body
            )));
        }

        serde_json::from_str(&body).map_err(|e| AppError::JsonParseError(e.to_string()))
    }
}

#[async_trait]
impl BlockchainScanner for TronScanner {
    fn chain_name(&self) -> &str {
        "TRON"
    }

    async fn get_latest_block(&self) -> Result<u64, AppError> {
        let result = self
            .make_http_request("wallet/getnowblock", json!({}))
            .await?;
        let block_number = result["block_header"]["raw_data"]["number"]
            .as_u64()
            .ok_or_else(|| AppError::ParseError("Failed to parse block number".to_string()))?;

        Ok(block_number)
    }

    async fn scan_block(&self, block_num: u64) -> Result<Vec<Value>, AppError> {
        let params = json!({
            "num": block_num
        });

        let result = self
            .make_http_request("wallet/gettransactioninfobyblocknum", params)
            .await?;

        let transactions = result.as_array().ok_or_else(|| {
            AppError::ParseError(format!(
                "Failed to parse transactions for block {}",
                block_num
            ))
        })?;

        let mut parsed_transactions = Vec::new();
        for transaction in transactions.iter() {
            match self.parse_transaction(transaction, &Value::Null).await {
                Ok(parsed_tx) => parsed_transactions.push(parsed_tx),
                Err(e) => println!("Error parsing transaction: {:?}", e),
            }
        }

        Ok(parsed_transactions)
    }

    async fn filter_transactions(&self, transactions: Vec<Value>) -> Vec<Value> {
        transactions
    }

    async fn parse_transaction(
        &self,
        transaction: &Value,
        _receipt: &Value,
    ) -> Result<Value, AppError> {
        println!("Raw transaction: {:?}", transaction);

        let tx_id = transaction["id"]
            .as_str()
            .ok_or_else(|| AppError::ParseError("Missing transaction id".to_string()))?;

        let block_number = transaction["blockNumber"]
            .as_u64()
            .ok_or_else(|| AppError::ParseError("Invalid block number".to_string()))?;

        let fee = transaction["receipt"]
            .get("net_fee")
            .or_else(|| transaction.get("fee"))
            .and_then(|f| f.as_u64())
            .unwrap_or(0);

        let energy_usage_total = transaction
            .get("receipt")
            .and_then(|r| r.get("energy_usage_total"))
            .and_then(|e| e.as_u64())
            .unwrap_or(0);

        let status = transaction
            .get("receipt")
            .and_then(|r| r.get("result"))
            .and_then(|s| s.as_str())
            .unwrap_or("success");

        let contract_address = transaction
            .get("contract_address")
            .and_then(|a| a.as_str())
            .map(|s| hex_to_tron_address(s).unwrap_or_else(|_| s.to_string()))
            .unwrap_or_default();

        let mut parsed_tx = json!({
            "tx_hash": tx_id,
            "block_number": block_number,
            "fee": fee,
            "energy_usage_total": energy_usage_total,
            "status": status,
            "contract_address": contract_address,
        });

        if let Some(contract) = transaction
            .get("raw_data")
            .and_then(|rd| rd.get("contract"))
            .and_then(|c| c.get(0))
        {
            if let Some(parameter) = contract.get("parameter").and_then(|p| p.get("value")) {
                if let Some(from) = parameter.get("owner_address").and_then(|a| a.as_str()) {
                    parsed_tx["from"] = json!(hex_to_tron_address(&format!("41{}", &from))
                        .unwrap_or_else(|_| from.to_string()));
                }
                if let Some(to) = parameter.get("to_address").and_then(|a| a.as_str()) {
                    parsed_tx["to"] = json!(hex_to_tron_address(&format!("41{}", &to))
                        .unwrap_or_else(|_| to.to_string()));
                }
                if let Some(value) = parameter.get("amount").and_then(|a| a.as_u64()) {
                    parsed_tx["value"] = json!(value);
                }
            }
        }

        if let Some(logs) = transaction.get("log").and_then(|l| l.as_array()) {
            let mut token_transfers = Vec::new();
            for log in logs {
                if let Some(topics) = log.get("topics").and_then(|t| t.as_array()) {
                    if topics.len() >= 3
                        && topics[0].as_str()
                            == Some(
                                "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                            )
                    {
                        let from = topics
                            .get(1)
                            .and_then(|t| t.as_str())
                            .and_then(|s| hex_to_tron_address(&format!("41{}", &s[24..])).ok())
                            .unwrap_or_else(|| "Unknown".to_string());
                        let to = topics
                            .get(2)
                            .and_then(|t| t.as_str())
                            .and_then(|s| hex_to_tron_address(&format!("41{}", &s[24..])).ok())
                            .unwrap_or_else(|| "Unknown".to_string());
                        let value = log
                            .get("data")
                            .and_then(|d| d.as_str())
                            .map(|s| {
                                BigInt::from_str_radix(s.trim_start_matches("0x"), 16)
                                    .unwrap_or(BigInt::from(0))
                            })
                            .unwrap_or(BigInt::from(0));

                        token_transfers.push(json!({
                            "from": from,
                            "to": to,
                            "value": value.to_string()
                        }));
                    }
                }
            }
            if !token_transfers.is_empty() {
                parsed_tx["token_transfers"] = json!(token_transfers);
            }
        }

        Ok(parsed_tx)
    }

    fn get_start_block(&self) -> u64 {
        self.start_block
    }
}
