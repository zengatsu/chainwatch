
use std::collections::VecDeque;


pub struct AppState {
    pub last_blocks: VecDeque<u64>,
    pub last_txs: VecDeque<Transaction>,
    pub total_blocks: u64,
    pub total_txs: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            last_blocks: VecDeque::with_capacity(10),
            last_txs: VecDeque::with_capacity(10),
            total_blocks: 0,
            total_txs: 0,
        }
    }

    pub fn print(self: &Self) {
        for tx in &self.last_txs {
            println!("block: {:?}, tx: {:?}, {:?} -> {:?}", &self.last_blocks.iter().last().unwrap_or(&0), tx.hash, tx.from, tx.to);
        }
    }
}

#[derive(Clone)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub is_whale: bool,
    pub is_stylus: bool,
}