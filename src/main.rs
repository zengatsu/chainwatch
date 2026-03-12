mod service;
mod state;
mod tui;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use alloy::primitives::U256;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
};
use dotenv::dotenv;
use eyre::Result;

use crate::{
    service::{get_block_data, stream_transactions},
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    dotenv().ok();

    // We should have cli mode
    //  - show the transaction in a block or latest block
    //  - stream mode shows the transactions as they come in
    // Tui mode
    //  - show the transactions and all stats as they come in

    let threshold_value = cli.threshold.unwrap_or(10);
    let threshold = U256::from(threshold_value)
        .checked_mul(U256::from(threshold_value).pow(U256::from(18)))
        .unwrap();

    let state = Arc::new(Mutex::new(AppState::new()));
    let fetch_state = Arc::clone(&state);

    if cli.stream {
        let mut terminal = tui::setup_tui()?;
        terminal.draw(|f| {
            tui::draw(f, &state);
        })?;

        stream_transactions(threshold, fetch_state).await?;
        ///////////////////////////////////
        loop {
            terminal.draw(|f| {
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

        tui::reset()?;

        return Ok(());
    }

    get_block_data(cli.block).await?;

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
}
