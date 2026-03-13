use std::{
    env,
    sync::{Arc, Mutex},
};

use alloy::{
    consensus::Transaction,
    network::{AnyNetwork, TransactionResponse},
    primitives::U256,
    providers::{Provider, ProviderBuilder, WsConnect},
};
use eyre::Result;
use tokio_stream::StreamExt;

use crate::state::AppState;

pub async fn stream_transactions(threshold: U256, fetch_state: Arc<Mutex<AppState>>, ui: Option<bool>) -> Result<()> {
    let ws_url = env::var("WS_URL").expect("WS_URL must be set in .env file!");
    let ws = WsConnect::new(ws_url);
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .connect_ws(ws)
        .await?;
    // println!("Connected! Waiting for new Arbitrum blocks...");

    // 3. Subscribe to new block headers
    let subscription = provider.subscribe_blocks().await?;
    let mut stream = subscription.into_stream();

    // 4. Loop forever as new blocks arrive
    tokio::spawn(async move {
        while let Some(header) = stream.next().await {
            let response = provider.get_block_by_hash(header.hash).full().await;

            if let Ok(Some(block)) = response {

                let mut sum = U256::from(0);
                let mut txs = Vec::<String>::new();

                // Get the transaction for this block
                for tx in block.transactions.as_transactions().unwrap() {
                    let value_wei = tx.value();
                    if value_wei >= threshold {
                        // TODO: add this to the state
                    }
                    sum += value_wei;
                    txs.push(tx.tx_hash().to_string());
                }

                // if not in tui mode print the stream
                if !ui.unwrap_or(false) {
                    for tx in &txs {
                        println!("block: {:?}, tx: {:?}", header.number,  tx);
                    }
                }

                let mut s = fetch_state.lock().unwrap();
                s.total_blocks += 1;
                s.total_txs += s.last_txs.len() as u64;
                s.last_blocks.push_front(header.number);
                s.last_txs.extend(txs);
                let excess = s.last_txs.len().saturating_sub(10);
                if excess > 0 {
                    s.last_txs.drain(0..excess);
                }
                if s.last_blocks.len() > 10 {
                    s.last_blocks.pop_back();
                }
            }
        }
    });

    Ok(())
}

pub async fn get_block_data(block_number: Option<u64>, fetch_state: Arc<Mutex<AppState>>) -> Result<()> {
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set in .env file!");
    let url = rpc_url.parse()?;
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .connect_http(url);

    let bn = match block_number {
        Some(bn) => bn,
        None => provider.get_block_number().await?,
    };


    let block = provider
        .get_block_by_number(bn.into())
        .full()
        .await?
        .ok_or_else(|| eyre::eyre!("Block not found"))?;

    let mut s = fetch_state.lock().unwrap();

    let mut txs = Vec::<String>::new();
    for tx in block.transactions.as_transactions().unwrap() {
        txs.push(tx.tx_hash().to_string());
    }

    s.total_blocks = 1;
    s.total_txs = txs.len() as u64;
    s.last_blocks.push_front(bn);
    s.last_txs = txs.into();

    Ok(())
}