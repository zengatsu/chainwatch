use std::env;

use alloy::{consensus::Transaction, network::{AnyNetwork, TransactionResponse}, primitives::U256, providers::{Provider, ProviderBuilder, WsConnect}};
use clap::Parser;
use dotenv::dotenv;
use eyre::Result;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    dotenv().ok();

    let threshold_value = cli.threshold.unwrap_or(10);
    let threshold = U256::from(threshold_value).checked_mul(U256::from(threshold_value).pow(U256::from(18))).unwrap();

    if let Some(block) = cli.block {
        println!("block: {}", block);
    } else {
        println!("block: Not specified");
    }

    if cli.stream {
        let ws_url = env::var("WS_URL").expect("WS_URL must be set in .env file!");
        let ws = WsConnect::new(ws_url);
        let provider = ProviderBuilder::new().network::<AnyNetwork>().connect_ws(ws).await?;
        println!("Connected! Waiting for new Arbitrum blocks...");


        // 3. Subscribe to new block headers
        let subscription = provider.subscribe_blocks().await?;
        let mut stream = subscription.into_stream();

        // 4. Loop forever as new blocks arrive
        while let Some(header) = stream.next().await {
            println!("New Block Detected!");
            println!("  Hash:   {:?}", header.hash);
            println!("  Number: {:?}", header.number);

            let block = provider.get_block_by_hash(header.hash).full().await?.ok_or_else(|| eyre::eyre!("Block details missing"))?;

            println!("Number of transactions: {:?}", block.transactions.hashes().len());

            let mut sum = U256::from(0);

            // Get the transaction for this block
            for tx in block.transactions.as_transactions().unwrap() {
                let value_wei = tx.value();
                if value_wei >= threshold {
                    print!("🚨 ");
                }
                println!("{:?}: {:?} - {:?}", tx.tx_hash(), tx.from(), tx.to().unwrap_or_default());
                sum += value_wei;
            }
            println!("Sum value of the block: {:?}", sum.checked_div(U256::from(10).pow(U256::from(18))).unwrap_or_default());
            println!("----------------------------------");
        }

        return Ok(());
    }

    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set in .env file!");
    let url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().network::<AnyNetwork>().connect_http(url);

    let block_number = match cli.block {
        Some(bn) => bn,
        None => provider.get_block_number().await?
    };

    println!("Fetching transactions for Arbitrum block: {}", block_number);

    let block = provider.get_block_by_number(block_number.into()).full().await?.ok_or_else(|| eyre::eyre!("Block not found"))?;

    for tx in block.transactions.as_transactions().unwrap() {
        println!("{:?}: {:?} - {:?}", tx.tx_hash(), tx.from(), tx.to());
    }

    Ok(())
}
// Define the structure for command-line arguments
#[derive(Parser, Debug)]
// Optional: add metadata for the generated help message
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    stream: bool,

    #[arg(short, long)]
    block: Option<u64>,
    
    #[arg(short, long)]
    threshold: Option<u64>,
}

