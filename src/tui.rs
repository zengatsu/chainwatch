use std::{
    io::Stdout, sync::{Arc, Mutex}
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;

use crate::state::AppState;

pub fn setup_tui() -> Result<Terminal<CrosstermBackend<Stdout>>> {

        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(terminal)
}

pub fn reset() -> Result<()> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

pub fn draw(
    f: &mut ratatui::Frame<'_>,
    state: &Arc<Mutex<AppState>>,
) -> std::rc::Rc<[ratatui::prelude::Rect]> {
    let size = f.area();
    // 1. Create a layout (Split screen into 3 sections)
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),      // Stats header
            ratatui::layout::Constraint::Percentage(40), // Blocks
            ratatui::layout::Constraint::Percentage(50), // Transactions
        ])
        .split(size);

    // 2. Render Stats
    let state = state.lock().unwrap();
    let stats = Paragraph::new(format!(
        "Total Blocks: {} | Total TXs: {}",
        state.total_blocks, state.total_txs
    ))
    .block(Block::default().title("Stats").borders(Borders::ALL));
    f.render_widget(stats, chunks[0]);

    // 3. Render Blocks List
    let blocks: Vec<ListItem> = state
        .last_blocks
        .iter()
        .map(|b| ListItem::new(format!("Block #{}", b)))
        .collect();
    let block_list = List::new(blocks).block(
        Block::default()
            .title("Recent Blocks")
            .borders(Borders::ALL),
    );
    f.render_widget(block_list, chunks[1]);
    // 3. Render transactions List
    let transactions: Vec<ListItem> = state
        .last_txs
        .iter()
        .map(|b| ListItem::new(format!("Tx: {}", b)))
        .collect();
    let transaction_list = List::new(transactions).block(
        Block::default()
            .title("Recent Blocks")
            .borders(Borders::ALL),
    );
    f.render_widget(transaction_list, chunks[2]);

    chunks
}
