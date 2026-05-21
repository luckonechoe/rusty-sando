use crate::types::{Collector, CollectorStream};
use alloy::primitives::{B256, U256};
use alloy::providers::Provider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio_stream::StreamExt;

/// A collector that listens for new blocks, and generates a stream of
/// [events](NewBlock) which contain the block number and hash.
pub struct BlockCollector<P> {
    provider: Arc<P>,
}

/// A new block event, containing the block number and hash.
#[derive(Debug, Clone)]
pub struct NewBlock {
    pub hash: B256,
    pub number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    /// EIP-1559 base fee. May be missing for pre-1559 chains; expressed as U256 for downstream math.
    pub base_fee_per_gas: U256,
    pub timestamp: u64,
}

impl<P> BlockCollector<P> {
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }
}

/// Implementation of the [Collector](Collector) trait for the [BlockCollector](BlockCollector).
/// Subscribes to `eth_subscribe("newHeads")` via an alloy pub-sub provider.
#[async_trait]
impl<P> Collector<NewBlock> for BlockCollector<P>
where
    P: Provider + 'static,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, NewBlock>> {
        let sub = self
            .provider
            .subscribe_blocks()
            .await
            .map_err(|e| anyhow::anyhow!("subscribe_blocks failed: {e}"))?;
        let stream = sub.into_stream().map(|header| NewBlock {
            hash: header.hash,
            number: header.number,
            gas_used: header.gas_used,
            gas_limit: header.gas_limit,
            base_fee_per_gas: U256::from(header.base_fee_per_gas.unwrap_or_default()),
            timestamp: header.timestamp,
        });
        Ok(Box::pin(stream))
    }
}
