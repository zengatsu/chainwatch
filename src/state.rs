use std::collections::VecDeque;

use ratatui::widgets::{ScrollbarState, TableState};
use sqlx::FromRow;

pub struct AppState {
    pub last_blocks: VecDeque<u64>,
    pub last_txs: VecDeque<Transaction>,
    pub total_blocks: u64,
    pub total_txs: u64,
    cached_txs: Vec<Transaction>,
    pub cached_t_state: TableState,
    pub cached_sb_state: ScrollbarState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            last_blocks: VecDeque::with_capacity(10),
            last_txs: VecDeque::with_capacity(10),
            total_blocks: 0,
            total_txs: 0,
            cached_txs: vec![],
            cached_t_state: TableState::default().with_selected(0),
            cached_sb_state: ScrollbarState::new(0),
        }
    }

    pub fn print(self: &Self) {
        for tx in &self.last_txs {
            println!(
                "block: {:?}, tx: {:?}, {:?} -> {:?}",
                &self.last_blocks.iter().last().unwrap_or(&0),
                tx.tx_hash,
                tx.from_addr,
                tx.to_addr
            );
        }
    }

    pub fn next(&mut self) {
        let i = match self.cached_t_state.selected() {
            Some(i) => {
                if i >= self.cached_txs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.cached_t_state.select(Some(i));
        // Sync scrollbar position with the table selection
        self.cached_sb_state = self.cached_sb_state.position(i);
    }

    pub fn previous(&mut self) {
        let i = match self.cached_t_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.cached_txs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.cached_t_state.select(Some(i));
        self.cached_sb_state = self.cached_sb_state.position(i);
    }

    pub fn cached_txs(&self) -> &[Transaction] {
        &self.cached_txs
    }

    pub fn set_cached_txs(&mut self, cached_txs: Vec<Transaction>) {
        self.cached_txs = cached_txs;
        self.cached_sb_state = self.cached_sb_state.content_length(self.cached_txs.len());
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct Transaction {
    pub block_number: u64,
    pub tx_hash: String,
    pub from_addr: String,
    pub to_addr: String,
    pub eth_value: String,
    pub is_whale: bool,
    pub is_stylus: bool,
}
