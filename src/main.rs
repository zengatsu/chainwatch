mod service;
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

use crate::{
    service::{get_block_data, stream_transactions},
    state::{AppState, Transaction},
};

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
    tokio::spawn(async move {
        while let Some(tx) = tx_receiver.recv().await {
            let value_str = tx.value.to_string();
            let block = tx.block as i64;
            sqlx::query!(
                "INSERT INTO transactions (block_number, tx_hash, from_addr, to_addr, eth_value, is_whale, is_stylus) 
                VALUES (?, ?, ?, ?, ?, ?, ?)",
                block, tx.hash, tx.from, tx.to, value_str, tx.is_whale, tx.is_stylus
            )
            .execute(&pool)
            .await
            .ok();
        }
    });

    let state = Arc::new(Mutex::new(AppState::new()));
    let fetch_state = Arc::clone(&state);

    if cli.stream {
        stream_transactions(threshold, fetch_state, tx_sender, Some(cli.ui)).await?;

        // Initialize terminal using .then() for a more functional approach
        let mut terminal = cli.ui.then(|| tui::setup_tui(Some(true))).transpose()?;

        loop {
            // Use .as_mut() to interact with the terminal only if it exists
            if let Some(t) = terminal.as_mut() {
                t.draw(|f| {
                    tui::draw(f, &state);
                })?;

                // Check for "Q" key to quit
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Char('q') {
                            break;
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
            tui::draw(f, &state);
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
