use std::collections::VecDeque;

use sqlx::FromRow;

pub struct AppState {
    pub last_blocks: VecDeque<u64>,
    pub last_txs: VecDeque<Transaction>,
    pub total_blocks: u64,
    pub total_txs: u64,
    pub chached_txs: Vec<Transaction>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            last_blocks: VecDeque::with_capacity(10),
            last_txs: VecDeque::with_capacity(10),
            total_blocks: 0,
            total_txs: 0,
            chached_txs: vec![],
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
