use crate::scanners::BlockchainScanner;
use crate::utils::error::AppError;
use async_trait::async_trait;
use num_bigint::BigInt;
use num_traits::Num;
use reqwest::Client;
use serde_json::{json, Value};

pub struct BscScanner {
    api_url: String,
    client: Client,
    start_block: u64,
}

impl BscScanner {
    pub fn new(api_url: &str, start_block: u64) -> Result<Self, AppError> {
        Ok(BscScanner {
            api_url: api_url.to_string(),
            client: Client::new(),
            start_block,
        })
    }

    async fn make_request(&self, method: &str, params: Value) -> Result<Value, AppError> {
        let response = self
            .client
            .post(&self.api_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            }))
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?
            .json::<Value>()
            .await
            .map_err(|e| AppError::JsonParseError(e.to_string()))?;

        if let Some(error) = response.get("error") {
            return Err(AppError::ApiError(error.to_string()));
        }

        Ok(response["result"].clone())
    }
}

#[async_trait]
impl BlockchainScanner for BscScanner {
    fn chain_name(&self) -> &str {
        "BSC"
    }

    async fn get_latest_block(&self) -> Result<u64, AppError> {
        let result = self.make_request("eth_blockNumber", json!([])).await?;
        let block_number =
            u64::from_str_radix(result.as_str().unwrap().trim_start_matches("0x"), 16).map_err(
                |e| AppError::ParseError(format!("Failed to parse block number: {}", e)),
            )?;
        Ok(block_number)
    }

    async fn scan_block(&self, block_num: u64) -> Result<Vec<Value>, AppError> {
        let block_hex = format!("0x{:X}", block_num);
        let result = self
            .make_request("qn_getBlockWithReceipts", json!([block_hex]))
            .await?;

        // 检查结果是否为对象
        if !result.is_object() {
            return Err(AppError::ParseError("Result is not an object".to_string()));
        }

        // 从 "block" 字段中获取 transactions
        let block = result
            .get("block")
            .and_then(|b| b.as_object())
            .ok_or_else(|| AppError::ParseError("Failed to parse block".to_string()))?;

        let transactions = block
            .get("transactions")
            .and_then(|t| t.as_array())
            .ok_or_else(|| AppError::ParseError("Failed to parse transactions".to_string()))?;

        // 获取 receipts
        let receipts = result
            .get("receipts")
            .and_then(|t| t.as_array())
            .ok_or_else(|| AppError::ParseError("Failed to parse receipts".to_string()))?;

        // 确保 transactions 和 receipts 数量匹配
        if transactions.len() != receipts.len() {
            return Err(AppError::ParseError(
                "Mismatch between transactions and receipts".to_string(),
            ));
        }

        // 逐个解析交易
        let mut parsed_transactions = Vec::new();
        for (transaction, receipt) in transactions.iter().zip(receipts.iter()) {
            let parsed_tx = self.parse_transaction(transaction, receipt).await?;
            parsed_transactions.push(parsed_tx);
        }

        Ok(parsed_transactions)
    }

    async fn filter_transactions(&self, transactions: Vec<Value>) -> Vec<Value> {
        // 实现过滤BSC交易的逻辑
        // 这里只是一个示例，您需要根据实际需求实现
        transactions
    }

    async fn parse_transaction(
        &self,
        transaction: &Value,
        receipt: &Value,
    ) -> Result<Value, AppError> {
        // 获取常见的交易字段
        let tx_hash = transaction
            .get("hash")
            .and_then(|t| t.as_str())
            .ok_or_else(|| AppError::ParseError("Missing transaction hash".to_string()))?;

        let block_number_hex = transaction
            .get("blockNumber")
            .and_then(|t| t.as_str())
            .ok_or_else(|| AppError::ParseError("Missing block number".to_string()))?;

        // 将block_number从十六进制转换为十进制
        let block_number = u64::from_str_radix(&block_number_hex.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::ParseError("Failed to parse block number".to_string()))?;

        let from_address = transaction
            .get("from")
            .and_then(|t| t.as_str())
            .ok_or_else(|| AppError::ParseError("Missing from address".to_string()))?;

        let to_address = transaction
            .get("to")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown"); // 'to' 可能为空

        let gas_used = receipt
            .get("gasUsed")
            .and_then(|t| t.as_str())
            .unwrap_or("0"); // 从 receipt 获取 gasUsed

        let value_hex = transaction
            .get("value")
            .and_then(|t| t.as_str())
            .unwrap_or("0x0"); // 如果 value 为空，设置为默认的 "0x0"

        // 尝试将 value 从十六进制转换为 BigInt
        let value =
            BigInt::from_str_radix(&value_hex.trim_start_matches("0x"), 16).map_err(|e| {
                println!("Failed to parse value_hex: {}", value_hex); // 打印错误的值
                AppError::ParseError(format!("Failed to parse value: {}", e))
            })?;

        // 将 BigInt 转换为字符串
        let value_string = value.to_string();

        // 获取交易状态和确认数
        let status_hex = receipt
            .get("status")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        // 将交易状态从十六进制转换为十进制
        let status = match u64::from_str_radix(&status_hex.trim_start_matches("0x"), 16) {
            Ok(1) => "success",
            Ok(0) => "failed",
            _ => "unknown",
        };

        let confirmations = receipt
            .get("confirmations")
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        // 创建一个空的 Vec 作为默认值
        let empty_logs: Vec<Value> = Vec::new();
        let logs = receipt
            .get("logs")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty_logs);

        let is_contract = !logs.is_empty(); // 如果有 logs，说明是合约交互

        // 构建解析后的交易数据
        let mut parsed_tx = json!({
            "tx_hash": tx_hash,
            "block_number": block_number,
            "from": from_address,
            "to": to_address,
            "gas_used": gas_used,
            "value": value_string,  // 使用字符串形式的 value
            "status": status,
            "confirmations": confirmations,
            "is_contract": is_contract
        });

        // 如果是合约交易，解析 logs 中的 token_transfer
        if is_contract {
            let token_transfers: Vec<Value> = logs
                .iter()
                .filter(|log| {
                    let empty_topics: Vec<Value> = Vec::new();
                    let topics = log
                        .get("topics")
                        .and_then(|t| t.as_array())
                        .unwrap_or(&empty_topics);

                    topics.get(0).map_or(false, |topic| {
                        topic == "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                    })
                })
                .map(|log| {
                    // 解析 topics[1] 为 'from' 地址
                    let from = log
                        .get("topics")
                        .and_then(|t| t.get(1)) // 提取第一个参数 (from 地址)
                        .and_then(|address| address.as_str()) // 将 JSON 值解析为字符串
                        .map(|address| format!("0x{}", &address[26..])) // 处理地址，将地址后40个字符提取出来
                        .unwrap_or("Unknown from address".to_string());

                    // 解析 topics[2] 为 'to' 地址
                    let to = log
                        .get("topics")
                        .and_then(|t| t.get(2)) // 提取第二个参数 (to 地址)
                        .and_then(|address| address.as_str()) // 将 JSON 值解析为字符串
                        .map(|address| format!("0x{}", &address[26..])) // 处理地址，将地址后40个字符提取出来
                        .unwrap_or("Unknown to address".to_string());

                    // 修改 value 的解析
                    let value = log
                        .get("data")
                        .and_then(|v| v.as_str()) // 提取十六进制字符串
                        .map(|data| {
                            u128::from_str_radix(&data.trim_start_matches("0x"), 16).unwrap_or(0)
                        }) // 转换为十进制
                        .map(|v| v.to_string()) // 将 u128 转换为 String
                        .unwrap_or_else(|| "0".to_string()); // 如果 None，则使用 "0"

                    json!({
                        "from": from,
                        "to": to,
                        "value": value
                    })
                })
                .collect();

            parsed_tx["token_transfers"] = json!(token_transfers);
        }

        Ok(parsed_tx)
    }

    fn get_start_block(&self) -> u64 {
        self.start_block
    }
}
