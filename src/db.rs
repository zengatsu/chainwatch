use crate::state::Transaction;
use eyre::Result;
use sqlx::{Pool, Sqlite};

pub async fn get_cached_txs(pool: &Pool<Sqlite>) -> Result<Vec<Transaction>> {
    let res: Vec<Transaction> = sqlx::query_as::<_, Transaction>(
        "SELECT block_number, tx_hash, from_addr, to_addr, eth_value, is_whale, is_stylus
        FROM transactions",
    )
    .fetch_all(pool)
    .await?;
    Ok(res)
}
