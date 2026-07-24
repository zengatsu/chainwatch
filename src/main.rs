mod arbitrum;
mod db;
mod state;
mod tui;

use std::{
    env,
    sync::{Arc, Mutex},
};

use alloy::primitives::U256;
use clap::Parser;
use dotenv::dotenv;
use eyre::Result;

use arbitrum::{get_block_data, stream_transactions};
use state::{AppState, Transaction};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    dotenv().ok();

    let threshold_value = cli.threshold.unwrap_or(10);
    let threshold = U256::from(threshold_value)
        .checked_mul(U256::from(threshold_value).pow(U256::from(18)))
        .unwrap();

    let db_url = cli
        .db_url
        .unwrap_or(env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file!"));

    let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel::<Transaction>(100);
    let pool = sqlx::SqlitePool::connect(&db_url).await?;
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        while let Some(tx) = tx_receiver.recv().await {
            let value_str = tx.eth_value.to_string();
            let block = tx.block_number as i64;
            sqlx::query(
                "INSERT INTO transactions (block_number, tx_hash, from_addr, to_addr, eth_value, is_whale, is_stylus)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(block)
            .bind(&tx.tx_hash)
            .bind(&tx.from_addr)
            .bind(&tx.to_addr)
            .bind(&value_str)
            .bind(tx.is_whale)
            .bind(tx.is_stylus)
            .execute(&pool)
            .await
            .ok();
        }
    });

    let state = Arc::new(Mutex::new(AppState::new()));
    let fetch_state = Arc::clone(&state);
    let tab = 0;

    let txs = db::get_cached_txs(&pool_clone).await;
    state.lock().unwrap().set_cached_txs(txs?);

    if cli.stream {
        let ws_url = cli
            .ws
            .unwrap_or(env::var("WS_URL").expect("WS_URL must be set in .env file!"));
        stream_transactions(ws_url, threshold, fetch_state, tx_sender, Some(cli.ui)).await?;

        // Initialize terminal using .then() for a more functional approach
        let _ = cli.ui.then(|| tui::tui_loop(tab, state)).transpose()?;

        if cli.ui {
            tui::reset()?;
        }

        return Ok(());
    }

    let rpc_url = cli
        .rpc
        .unwrap_or(env::var("RPC_URL").expect("RPC_URL must be set in .env file!"));
    get_block_data(rpc_url, cli.block, fetch_state).await?;

    if cli.ui {
        let _ = tui::tui_loop(tab, state);

        if cli.ui {
            tui::reset()?;
        }
    } else {
        state.lock().unwrap().print();
    }

    Ok(())
}
// Define the structure for command-line arguments
#[derive(Parser, Debug)]
// Optional: add metadata for the generated help message
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    stream: bool,

    #[arg(short, long)]
    block: Option<u64>,

    #[arg(short, long)]
    threshold: Option<u64>,

    #[arg(short, long)]
    ui: bool,

    #[arg(short, long)]
    ws: Option<String>,

    #[arg(short, long)]
    rpc: Option<String>,

    #[arg(short, long)]
    db_url: Option<String>,
}
