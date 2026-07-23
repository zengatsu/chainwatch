use std::sync::{Arc, Mutex};

use alloy::{
    consensus::Transaction,
    network::{AnyNetwork, TransactionResponse},
    primitives::U256,
    providers::{Provider, ProviderBuilder, WsConnect},
};
use eyre::Result;
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

use crate::state::AppState;
use crate::state::Transaction as Tx;

pub async fn stream_transactions(
    ws_url: String,
    threshold: U256,
    fetch_state: Arc<Mutex<AppState>>,
    tx_sender: Sender<Tx>,
    ui: Option<bool>,
) -> Result<()> {
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
                let mut txs = Vec::<Tx>::new();

                // Get the transaction for this block
                for tx in block.transactions.as_transactions().unwrap() {
                    let value_wei = tx.value();
                    sum += value_wei;
                    let is_whale = value_wei >= threshold;

                    let is_stylus = if let Some(to_addr) = tx.to() {
                        if let Ok(code) = provider.get_code_at(to_addr).await {
                            code.starts_with(&[0xef, 0x00])
                                || code.starts_with(&[0x00, 0x61, 0x73, 0x6d])
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let trans = Tx {
                        tx_hash: tx.tx_hash().to_string(),
                        from_addr: tx.from().to_string(),
                        to_addr: tx.to().unwrap_or_default().to_string(),
                        is_whale: is_whale,
                        is_stylus: is_stylus,
                        block_number: tx.block_number().unwrap_or_default(),
                        eth_value: value_wei.to_string(),
                    };
                    if is_whale || is_stylus {
                        let _ = tx_sender.send(trans.clone()).await;
                    }
                    txs.push(trans);
                }

                // if not in tui mode print the stream
                if !ui.unwrap_or(false) {
                    for tx in &txs {
                        println!(
                            "block: {:?}, tx: {:?}, {:?} -> {:?}",
                            header.number, tx.tx_hash, tx.from_addr, tx.to_addr
                        );
                    }
                }

                let mut s = fetch_state.lock().unwrap();
                s.total_blocks += 1;
                s.total_txs += s.last_txs.len() as u64;
                s.last_blocks.push_front(header.number);
                s.append_last_txs(txs);
            }
        }
    });

    Ok(())
}

pub async fn get_block_data(
    rpc_url: String,
    block_number: Option<u64>,
    fetch_state: Arc<Mutex<AppState>>,
) -> Result<()> {
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

    let mut txs = Vec::<Tx>::new();
    for tx in block.transactions.as_transactions().unwrap() {
        txs.push(Tx {
            tx_hash: tx.tx_hash().to_string(),
            from_addr: tx.from().to_string(),
            to_addr: tx.to().unwrap_or_default().to_string(),
            is_whale: false,
            is_stylus: false,
            block_number: tx.block_number.unwrap_or_default(),
            eth_value: tx.value().to_string(),
        });
    }

    s.total_blocks = 1;
    s.total_txs = txs.len() as u64;
    s.last_blocks.push_front(bn);
    s.last_txs = txs.into();

    Ok(())
}
