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
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    dotenv().ok();

    let threshold_value = cli.threshold.unwrap_or(10);
    let threshold = U256::from(threshold_value)
        .checked_mul(U256::from(threshold_value).pow(U256::from(18)))
        .unwrap();

    let state = Arc::new(Mutex::new(AppState::new()));
    let fetch_state = Arc::clone(&state);

    if cli.stream {
        stream_transactions(threshold, fetch_state, Some(cli.ui)).await?;

        // Initialize terminal using .then() for a more functional approach
        let mut terminal = cli.ui.then(|| tui::setup_tui(Some(true))).transpose()?;

        loop {
            // Use .as_mut() to interact with the terminal only if it exists
            if let Some(t) = terminal.as_mut() {
                t.draw(|f| {tui::draw(f, &state);})?;

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
