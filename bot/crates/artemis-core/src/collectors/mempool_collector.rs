use crate::types::{Collector, CollectorStream};
use alloy::consensus::TxType;
use alloy::providers::Provider;
use alloy::rpc::types::Transaction;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio_stream::StreamExt;

/// A collector that listens for new pending transactions in the mempool and
/// generates a stream of full [Transaction] objects.
///
/// EIP-4844 blob transactions (type 3) are filtered out at the source: the
/// sandwich strategy can never extract MEV from blob carriers because their
/// payload is a KZG commitment, not calldata, and including one in a bundle
/// would just inflate gas without altering pool state.
pub struct MempoolCollector<P> {
    provider: Arc<P>,
}

impl<P> MempoolCollector<P> {
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider }
    }
}

/// Implementation of the [Collector](Collector) trait for [MempoolCollector].
/// Uses `alloy_pubsub::Subscription` over `eth_subscribe("newPendingTransactions", true)`
/// (full bodies) so we can inspect the transaction type before forwarding.
#[async_trait]
impl<P> Collector<Transaction> for MempoolCollector<P>
where
    P: Provider + 'static,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Transaction>> {
        let sub = self
            .provider
            .subscribe_full_pending_transactions()
            .await
            .map_err(|e| anyhow::anyhow!("subscribe_full_pending_transactions failed: {e}"))?;
        // Drop EIP-4844 blob carriers (type 3); they can't be sandwiched.
        let stream = sub
            .into_stream()
            .filter(|tx| tx.inner.tx_type() != TxType::Eip4844);
        Ok(Box::pin(stream))
    }
}
