//! Integration tests for artemis-core collectors.
//!
//! These tests are gated behind the `live` feature because they require an
//! Anvil binary on PATH. Build them with `cargo test --features live` once
//! Phase 3 wires Anvil into CI.

#![cfg(feature = "live")]

use alloy::node_bindings::{Anvil, AnvilInstance};
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::TransactionRequest;
use alloy::primitives::{Address, U256};
use artemis_core::{
    collectors::{block_collector::BlockCollector, mempool_collector::MempoolCollector},
    types::Collector,
};
use std::sync::Arc;
use tokio_stream::StreamExt;

async fn spawn_anvil() -> (impl Provider<alloy::pubsub::PubSubFrontend>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = ProviderBuilder::new()
        .on_ws(WsConnect::new(anvil.ws_endpoint()))
        .await
        .unwrap();
    (provider, anvil)
}

#[tokio::test]
async fn test_block_collector_sends_blocks() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let block_collector = BlockCollector::new(provider.clone());
    let block_stream = block_collector.get_event_stream().await.unwrap();
    let block_a = block_stream.into_future().await.0.unwrap();
    let latest = provider.get_block_number().await.unwrap();
    assert!(block_a.number <= latest);
}

#[tokio::test]
async fn test_mempool_collector_sends_txs() {
    let (provider, _anvil) = spawn_anvil().await;
    let provider = Arc::new(provider);
    let mempool_collector = MempoolCollector::new(provider.clone());
    let mempool_stream = mempool_collector.get_event_stream().await.unwrap();

    let accounts = provider.get_accounts().await.unwrap();
    let from: Address = accounts[0];
    let value = U256::from(42u64);
    let tx = TransactionRequest::default()
        .to(from)
        .from(from)
        .value(value);

    let _pending = provider.send_transaction(tx).await.unwrap();
    let tx = mempool_stream.into_future().await.0.unwrap();
    assert_eq!(tx.value, value);
}
