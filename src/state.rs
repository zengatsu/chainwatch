use std::collections::VecDeque;

use ratatui::widgets::{ScrollbarState, TableState};
use sqlx::FromRow;

pub struct AppState {
    pub last_blocks: VecDeque<u64>,
    pub last_txs: VecDeque<Transaction>,
    pub total_blocks: u64,
    pub total_txs: u64,
    cached_txs: VecDeque<Transaction>,
    pub cached_t_state: TableState,
    pub cached_sb_state: ScrollbarState,
    pub stream_t_state: TableState,
    pub stream_sb_state: ScrollbarState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            last_blocks: VecDeque::with_capacity(10),
            last_txs: VecDeque::with_capacity(10),
            total_blocks: 0,
            total_txs: 0,
            cached_txs: VecDeque::with_capacity(1000),
            cached_t_state: TableState::default().with_selected(0),
            cached_sb_state: ScrollbarState::new(0),
            stream_t_state: TableState::default().with_selected(0),
            stream_sb_state: ScrollbarState::new(0),
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

    pub fn next(&mut self, active_tab: usize) {
        let (t_state, _, txs) = match active_tab {
            0 => (
                &mut self.stream_t_state,
                self.stream_sb_state,
                &self.last_txs,
            ),
            _ => (
                &mut self.cached_t_state,
                self.cached_sb_state,
                &self.cached_txs,
            ),
        };

        let i = match t_state.selected() {
            Some(i) => {
                if i >= txs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        t_state.select(Some(i));
        // self.cached_t_state.select(Some(i));
        // Sync scrollbar position with the table selection
        // self.cached_sb_state = self.cached_sb_state.position(i);
    }

    pub fn previous(&mut self, active_tab: usize) {
        let (mut t_state, _, txs) = match active_tab {
            0 => (
                &mut self.stream_t_state,
                self.stream_sb_state,
                &self.last_txs,
            ),
            _ => (
                &mut self.cached_t_state,
                self.cached_sb_state,
                &self.cached_txs,
            ),
        };

        let i = match t_state.selected() {
            Some(i) => {
                if i == 0 {
                    txs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        t_state.select(Some(i));
        // self.cached_t_state.select(Some(i));
        // self.cached_sb_state = self.cached_sb_state.position(i);
    }

    pub fn cached_txs(&self) -> &VecDeque<Transaction> {
        &self.cached_txs
    }

    pub fn set_cached_txs(&mut self, cached_txs: Vec<Transaction>) {
        self.cached_txs = cached_txs.into();
        self.cached_sb_state = self.cached_sb_state.content_length(self.cached_txs.len());
    }

    pub fn append_last_txs(&mut self, txs: Vec<Transaction>) {
        let prev_last = self.last_txs.len().saturating_sub(1);
        self.last_txs.extend(txs);

        // let excess = s.last_txs.len().saturating_sub(10);
        // if excess > 0 {
        //     s.last_txs.drain(0..excess);
        // }
        // if s.last_blocks.len() > 10 {
        //     s.last_blocks.pop_back();
        // }

        self.stream_sb_state = self.stream_sb_state.content_length(self.last_txs.len());
        match self.stream_t_state.selected() {
            Some(x) if x == prev_last => self.stream_t_state.select(Some(self.last_txs.len() - 1)),
            None => self.stream_t_state.select(Some(self.last_txs.len() - 1)),
            Some(_) => (),
        }
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
