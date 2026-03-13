use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Constraint,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    widgets::{Cell, Row, Table},
};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};

use crate::state::AppState;

pub fn setup_tui(raw_mode: Option<bool>) -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = std::io::stdout();

    if raw_mode.unwrap_or(false) {
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
    }

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
) {
    let size = f.area();

    let main_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(size);

    let content_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(20),
        ])
        .split(main_chunks[1]);

    // 2. Render Stats
    let state = state.lock().unwrap();
    let stats = Paragraph::new(format!(
        "Total Blocks: {} | Total TXs: {}",
        state.total_blocks, state.total_txs
    ))
    .block(Block::default().title("Stats").borders(Borders::ALL));
    f.render_widget(stats, main_chunks[0]);

    // 3. Render Blocks List
    let blocks: Vec<ListItem> = state
        .last_blocks
        .iter()
        .map(|b| ListItem::new(format!("{}", b)))
        .collect();
    let block_list = List::new(blocks).block(
        Block::default()
            .title("Recent Blocks")
            .borders(Borders::ALL),
    );
    f.render_widget(block_list, content_chunks[1]);

    // 3. Render transactions List
    let rows: Vec<Row> = state
        .last_txs
        .iter()
        .map(|tx| {
            Row::new(vec![
                Cell::from(tx.hash.to_string()),
                Cell::from(tx.from.to_string()),
                Cell::from(tx.to.to_string()),
                Cell::from(if tx.is_whale {"X"} else {""}),
                Cell::from(if tx.is_stylus {"X"} else {""}),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
        ],
    )
    .header(
        Row::new(vec!["Hash", "From", "To", "Whale", "Stylus"])
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title("Recent Transactions")
            .borders(Borders::ALL),
    )
    .column_spacing(1);

    f.render_widget(table, content_chunks[0]);

}
