mod arbitrum;
mod db;
mod state;
mod tui;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use alloy::primitives::U256;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
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

    let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel::<Transaction>(100);
    let pool = sqlx::SqlitePool::connect("sqlite:data/data.db").await?;
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        while let Some(tx) = tx_receiver.recv().await {
            let value_str = tx.eth_value.to_string();
            let block = tx.block_number as i64;
            sqlx::query!(
                "INSERT INTO transactions (block_number, tx_hash, from_addr, to_addr, eth_value, is_whale, is_stylus) 
                VALUES (?, ?, ?, ?, ?, ?, ?)",
                block, tx.tx_hash, tx.from_addr, tx.to_addr, value_str, tx.is_whale, tx.is_stylus
            )
            .execute(&pool)
            .await
            .ok();
        }
    });

    let state = Arc::new(Mutex::new(AppState::new()));
    let fetch_state = Arc::clone(&state);
    let mut tab = 0;

    let txs = db::get_cached_txs(&pool_clone).await;
    state.lock().unwrap().set_cached_txs(txs?);

    if cli.stream {
        stream_transactions(threshold, fetch_state, tx_sender, Some(cli.ui)).await?;

        // Initialize terminal using .then() for a more functional approach
        let mut terminal = cli.ui.then(|| tui::setup_tui(Some(true))).transpose()?;

        loop {
            // Use .as_mut() to interact with the terminal only if it exists
            if let Some(t) = terminal.as_mut() {
                t.draw(|f| {
                    tui::draw(f, tab, &state);
                })?;

                // Check for "Q" key to quit
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('l') | KeyCode::Right => tab = (tab + 1) % 2,
                            KeyCode::Char('h') | KeyCode::Left => tab = (tab + 2) % 2,
                            KeyCode::Up | KeyCode::Char('k') => state.lock().unwrap().previous(),
                            KeyCode::Down | KeyCode::Char('j') => state.lock().unwrap().next(),
                            _ => {}
                        }
                    }
                }
            }
        }

        if cli.ui {
            tui::reset()?;
        }

        return Ok(());
    }

    get_block_data(cli.block, fetch_state).await?;
    if cli.ui {
        let mut terminal = tui::setup_tui(None)?;
        terminal.draw(|f| {
            tui::draw(f, tab, &state);
        })?;
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
}
