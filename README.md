# XWScanner - Multi-Chain Block Scanner

XWScanner is a powerful and efficient multi-chain block scanner currently supporting TRON and BSC blockchains. It offers a robust solution for scanning blocks, parsing transactions, and providing flexible filtering capabilities, all while maintaining high performance and ease of use.

## Key Components

- `TronScanner`: Implements the TRON blockchain scanning logic.
- `BscScanner`: Implements the BSC blockchain scanning logic.
- `BlockchainScanner` trait: Defines the common interface for all blockchain scanners.
- Utility modules: Provide error handling, loggin

## Features

- **Multi-blockchain support**: Currently supports TRON and BSC, with easy extensibility for additional chains.
- **Efficient block scanning and transaction parsing**: Optimized for high-speed processing of large volumes of blockchain data.
- **Flexible transaction filtering mechanism**: Customizable filters to focus on specific types of transactions or smart contract interactions.
- **Asynchronous processing**: Leverages Rust's async capabilities for improved performance and responsiveness.
- **Configurable starting block and API endpoints**: Easily adjust scanning parameters through a centralized configuration file.
- **Elegant logging system**: Comprehensive logging with configurable levels for easy debugging and monitoring.
- **Error handling**: Robust error handling and reporting for reliable operation in production environments.
- **Modular architecture**: Well-structured codebase that's easy to understand, maintain, and extend.

## Advanced Features

- **Sophisticated configuration management**: Utilizes TOML for clear and flexible configuration, allowing easy adjustment of scanner behavior without code changes.
- **Cross-platform compatibility**: Runs seamlessly on various operating systems thanks to Rust's cross-platform capabilities.
- **Memory-efficient operation**: Designed with Rust's memory safety principles, ensuring efficient use of system resources even when processing large blockchain datasets.
- **Real-time monitoring capabilities**: Potential for integration with monitoring systems to provide up-to-date insights into blockchain activities.

## Quick Start

### Configuration

XWScanner uses a TOML configuration file located at `config/default.toml`. This file allows you to customize the behavior of the scanner for each supported blockchain.

Here's an example of the configuration file structure:
toml
[tron]
api_url = "https://api.trongrid.io"
start_block = 65766023
[bsc]
api_url = "https://bsc-dataseed.binance.org"
start_block = 20000000
[log]
level = "info"
file = "scanner.log"
[filter]
addresses = ["TRonAddressHere", "BSCAddressHere"]
contract_addresses = ["TRonContractAddressHere", "BSCContractAddressHere"]

Configuration options:

- `api_url`: The URL of the blockchain API endpoint.
- `start_block`: The block number from which to start scanning.
- `log.level`: The logging level (e.g., "debug", "info", "warn", "error").
- `log.file`: The file path for log output.
- `filter.addresses`: A list of addresses to monitor for transactions.
- `filter.contract_addresses`: A list of smart contract addresses to monitor for interactions.

Adjust these values according to your specific requirements before running the scanner.

### Prerequisites

- Rust 1.55.0 or higher
- Cargo

### Installation

1. Clone the repository:https://github.com/xwleo/block-scanner.git
