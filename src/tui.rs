use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;
use itertools::Itertools;
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
    time::Duration,
};

use crate::{state::AppState, tui};

const TABS: usize = 2;

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

pub fn tui_loop(mut tab: usize, state: Arc<Mutex<AppState>>) -> Result<()> {
    let mut terminal = tui::setup_tui(Some(true))?;

    loop {
        // Use .as_mut() to interact with the terminal only if it exists
        terminal.draw(|f| {
            tui::draw(f, tab, &state);
        })?;

        // Check for "Q" key to quit
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('l') | KeyCode::Right => tab = (tab + TABS + 1) % TABS,
                    KeyCode::Char('h') | KeyCode::Left => tab = (tab + TABS - 1) % TABS,
                    KeyCode::Up | KeyCode::Char('k') => state.lock().unwrap().previous(tab),
                    KeyCode::Down | KeyCode::Char('j') => state.lock().unwrap().next(tab),
                    _ => {}
                }
            }
        }
    }

    Ok(())
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
        0 => render_streaming_tab(f, content_chunk, &mut state),
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

fn render_streaming_tab(f: &mut Frame<'_>, area: Rect, state: &mut AppState) {
    let content_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(20),
        ])
        .split(area);

    let table_area = content_chunks[0];
    let side_area = content_chunks[1];

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
    f.render_widget(block_list, side_area);

    let rows: Vec<Row> = state
        .last_txs
        .iter()
        .chunk_by(|x| x.block_number)
        .into_iter()
        .flat_map(|(block_number, group)| {
            std::iter::once(Row::new(vec![Cell::from(format!("Block {block_number}"))])).chain(
                group.map(|tx| {
                    Row::new(vec![
                        Cell::from(tx.tx_hash.to_string()),
                        Cell::from(tx.from_addr.to_string()),
                        Cell::from(tx.to_addr.to_string()),
                        Cell::from(if tx.is_whale { "X" } else { "" }),
                        Cell::from(if tx.is_stylus { "X" } else { "" }),
                    ])
                }),
            )
        })
        .collect();

    let prev_rows_length = state.stream_rows_length.saturating_sub(1);
    state.stream_rows_length = rows.len();

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

    state.stream_sb_state = state
        .stream_sb_state
        .content_length(state.stream_rows_length);
    match state.stream_t_state.selected() {
        Some(x) if x == prev_rows_length => state
            .stream_t_state
            .select(Some(state.stream_rows_length.saturating_sub(1))),
        None => state
            .stream_t_state
            .select(Some(state.stream_rows_length.saturating_sub(1))),
        Some(_) => (),
    }

    f.render_stateful_widget(table, table_area, &mut state.stream_t_state);

    let viewport_length = table_area.height.saturating_sub(4) as usize;
    let content_length = state.stream_rows_length.saturating_sub(viewport_length);

    state.stream_sb_state = state
        .stream_sb_state
        .content_length(content_length)
        .viewport_content_length(viewport_length)
        .position(state.stream_t_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    f.render_stateful_widget(scrollbar, table_area, &mut state.stream_sb_state);
}

fn render_cached_txs(f: &mut Frame, table_area: Rect, state: &mut AppState) {
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

    f.render_stateful_widget(table, table_area, &mut state.cached_t_state);

    let viewport_length = table_area.height.saturating_sub(4) as usize;
    let content_length = state.cached_txs().len().saturating_sub(viewport_length);

    state.cached_sb_state = state
        .cached_sb_state
        .content_length(content_length)
        .viewport_content_length(viewport_length)
        .position(state.cached_t_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    f.render_stateful_widget(
        scrollbar,
        table_area.inner(Margin {
            vertical: 1, // Offset from top/bottom borders
            horizontal: 0,
        }),
        &mut state.cached_sb_state,
    );
}
