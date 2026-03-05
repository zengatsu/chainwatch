use std::env;

use alloy::{consensus::Transaction, network::{AnyNetwork, TransactionResponse}, providers::{Provider, ProviderBuilder}};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();

    let rpc_url = "https://arb1.arbitrum.io/rpc".parse()?;
    let provider = ProviderBuilder::new().network::<AnyNetwork>().connect_http(rpc_url);

    _ = args.next();
    let block_number = match args.next() {
        Some(bn) => {
            println!("{:?}", bn);
            bn.parse::<u64>()?
        },
        None => {
            let bn = provider.get_block_number().await?;
            bn
        }
    };

    println!("{:?}", block_number);

    println!("Fetching transactions for Arbitrum block: {}", block_number);

    let block = provider.get_block_by_number(block_number.into()).full().await?.ok_or_else(|| eyre::eyre!("Block not found"))?;

    for tx in block.transactions.as_transactions().unwrap() {
        println!("{:?}: {:?} - {:?}", tx.tx_hash(), tx.from(), tx.to());
    }

    Ok(())
}
