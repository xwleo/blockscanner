use bs58;
use sha2::{Digest, Sha256};

use super::error::AppError;

/// 将十六进制形式的 TRON 地址转换为 Base58 地址（以 'T' 开头）
pub fn hex_to_tron_address(hex: &str) -> Result<String, AppError> {
    let hex = hex.trim_start_matches("41");
    if hex.len() != 40 {
        return Err(AppError::ParseError(
            "Invalid Tron address length".to_string(),
        ));
    }

    let mut decoded = vec![0x41];
    decoded.extend_from_slice(&hex::decode(hex).map_err(|e| AppError::ParseError(e.to_string()))?);

    // 计算校验和
    let mut hasher = Sha256::new();
    hasher.update(&decoded);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    let hash2 = hasher.finalize();

    // 取前4个字节作为校验和
    let checksum = &hash2[..4];

    // 将原始数据和校验和拼接
    decoded.extend_from_slice(checksum);

    // 进行 Base58 编码
    let address = bs58::encode(decoded).into_string();

    Ok(address)
}

/// 将十六进制形式的 TRON 交易哈希转换为正确的格式
pub fn hex_to_tron_txhash(hex_hash: &str) -> Result<String, String> {
    // 移除可能的 "0x" 前缀
    let clean_hex = hex_hash.trim_start_matches("0x");

    // 确保哈希长度正确（32字节 = 64个十六进制字符）
    if clean_hex.len() != 64 {
        return Err("Invalid transaction hash length".to_string());
    }

    // 直接返回清理后的十六进制字符串
    Ok(clean_hex.to_string())
}
