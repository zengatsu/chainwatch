# Chainwatch

Chainwatch is a small Rust CLI/TUI tool for monitoring Arbitrum transactions and surfacing interesting activity such as whale transfers and Stylus-related interactions.

It can run in two modes:

- Streaming mode: subscribes to new blocks over WebSocket and watches live transactions.
- Snapshot mode: fetches a specific block (or the latest block) from an RPC endpoint.

## Features

- Live transaction streaming from an Arbitrum WebSocket endpoint
- Block inspection from an RPC endpoint
- Whale detection based on a configurable threshold
- Stylus contract detection for transaction recipients
- Optional terminal UI for browsing recent and cached transactions
- SQLite persistence for saved transactions

## Requirements

- Rust 1.85+ (the project currently targets the 2024 edition)
- A WebSocket endpoint for streaming
- An HTTP RPC endpoint for block fetching
- A SQLite database URL

## Getting started

Choose one of the following installation paths.

### Option 1: Use a release binary

This is the simplest path if a prebuilt binary is available for your platform.

1. Download the latest release archive from the Releases page for your OS.
2. Extract the archive and place the executable somewhere in your `PATH`, or run it directly from the extracted folder.
3. Create a `.env` file with the required endpoints and database URL
   ```dotenv
   DATABASE_URL=sqlite:chainwatch.db
   WS_URL=wss://your-arbitrum-ws-endpoint
   RPC_URL=https://your-arbitrum-rpc-endpoint
   ```
4. Initialize the SQLite table if needed

   The app expects a table named `transactions`. A starter schema is available in [src/migrations/_init.sql](src/migrations/_init.sql).

   Example:
   ```bash
   sqlite3 chainwatch.db < src/migrations/_init.sql
   ```
5. Run the app
   ```bash
   ./chainwatch --stream
   ```

### Option 2: Build from source

Use this if you want to compile from the current source tree or if no binary is available for your platform.

1. Install Rust 1.85+.
2. Clone the repository and build it
   ```bash
   git clone <repo-url>
   cd chainwatch
   cargo build --release
   ```
3. Create a `.env` file with the required endpoints and database URL
   ```dotenv
   DATABASE_URL=sqlite:data/chainwatch.db
   WS_URL=wss://your-arbitrum-ws-endpoint
   RPC_URL=https://your-arbitrum-rpc-endpoint
   ```
4. Initialize the SQLite table if needed

   Example:
   ```bash
   sqlite3 data/chainwatch.db < src/migrations/_init.sql
   ```
5. Run the built binary
   ```bash
   ./target/release/chainwatch --stream
   ```

## Command-line usage

### Common options

- `--stream`, `-s`: enable streaming mode over WebSocket
- `--block <BLOCK>`, `-b`: fetch a specific block number; if omitted in snapshot mode, the latest block is used
- `--threshold <THRESHOLD>`, `-t`: minimum transaction value to classify as a whale transfer. The value is interpreted as ETH and converted internally to wei
- `--ui`, `-u`: enable the terminal UI
- `--ws <WS_URL>`, `-w`: WebSocket URL override
- `--rpc <RPC_URL>`, `-r`: RPC URL override
- `--db-url <DB_URL>`, `-d`: SQLite database URL override

### Examples

Stream live transactions with the terminal UI:
```bash
chainwatch --stream --ui
```

Fetch the latest block and print the transactions:
```bash
chainwatch --block 12345678
```

Stream with a higher whale threshold:
```bash
chainwatch --stream --threshold 50
```

Use explicit endpoint overrides:
```bash
chainwatch --stream --ws wss://example.ws --rpc https://example.rpc --db-url sqlite:chainwatch.db
```

## How the app classifies transactions

Each transaction is evaluated for two markers:

- Whale transfer: the value is greater than or equal to the configured threshold
- Stylus interaction: the recipient address appears to contain Stylus bytecode

Transactions that match either condition are persisted to the SQLite database.

## Database

The app writes detected transactions into a SQLite table named `transactions` with the following columns:

- `block_number`
- `tx_hash`
- `from_addr`
- `to_addr`
- `eth_value`
- `is_whale`
- `is_stylus`
- `detected_at`

## TUI controls

When running with `--ui`, the interface supports:

- `Tab`/`Left`/`Right` to switch between views
- `j` / `k` or `Up` / `Down` to move through cached transactions
- `q` to quit

## Troubleshooting

- If the app exits with a missing environment variable error, make sure `.env` contains `DATABASE_URL`, `WS_URL`, and `RPC_URL`, or pass the values directly with the CLI flags.
- If the database table is missing, initialize it using [src/migrations/_init.sql](src/migrations/_init.sql).
- If the WebSocket or RPC connection fails, verify that the endpoint is reachable and supports the required network.
