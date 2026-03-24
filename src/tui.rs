use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Margin, Offset, Rect},
    style::{Color, Style},
    symbols,
    widgets::{
        Block, Borders, Cell, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        Table, Tabs,
    },
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

pub fn draw(f: &mut Frame<'_>, selected_tab: usize, state: &Arc<Mutex<AppState>>) {
    let size = f.area();
    let main_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(size);

    let tabs_chunk = main_chunks[0];
    let stats_chunk = main_chunks[1];
    let content_chunk = main_chunks[2];

    let mut state = state.lock().unwrap();

    let stats = Paragraph::new(format!(
        "Total Blocks: {} | Total TXs: {}",
        state.total_blocks, state.total_txs
    ))
    .block(Block::default().title("Stats").borders(Borders::ALL));
    f.render_widget(stats, stats_chunk);

    match selected_tab {
        0 => render_streaming_tab(f, content_chunk, &state),
        1 => render_cached_txs(f, content_chunk, &mut state),

        _ => unreachable!(),
    };

    render_tabs(f, tabs_chunk + Offset::new(1, 0), selected_tab);
}

pub fn render_tabs(f: &mut Frame, area: Rect, selected_tab: usize) {
    let tabs = Tabs::new(vec!["Transactions", "Saved Transactions"])
        .style(Color::White)
        .highlight_style(Style::default().magenta().on_black().bold())
        .select(selected_tab)
        .divider(symbols::DOT)
        .padding(" ", " ");
    f.render_widget(tabs, area);
}

fn render_streaming_tab(f: &mut Frame<'_>, main_chunk: Rect, state: &AppState) {
    let content_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(20),
        ])
        .split(main_chunk);

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
                Cell::from(tx.tx_hash.to_string()),
                Cell::from(tx.from_addr.to_string()),
                Cell::from(tx.to_addr.to_string()),
                Cell::from(if tx.is_whale { "X" } else { "" }),
                Cell::from(if tx.is_stylus { "X" } else { "" }),
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

fn render_cached_txs(f: &mut Frame, main: Rect, state: &mut AppState) {
    // 3. Render transactions List
    let rows: Vec<Row> = state
        .cached_txs()
        .iter()
        .map(|tx| {
            Row::new(vec![
                Cell::from(tx.tx_hash.to_string()),
                Cell::from(tx.from_addr.to_string()),
                Cell::from(tx.to_addr.to_string()),
                Cell::from(if tx.is_whale { "X" } else { "" }),
                Cell::from(if tx.is_stylus { "X" } else { "" }),
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
    .column_spacing(1)
    .row_highlight_style(
        ratatui::style::Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)
            .add_modifier(ratatui::style::Modifier::BOLD),
    );

    f.render_stateful_widget(table, main, &mut state.cached_t_state);
    state.cached_sb_state = state
        .cached_sb_state
        .position(state.cached_t_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    f.render_stateful_widget(
        scrollbar,
        main.inner(Margin {
            vertical: 1, // Offset from top/bottom borders
            horizontal: 0,
        }),
        &mut state.cached_sb_state,
    );
}
