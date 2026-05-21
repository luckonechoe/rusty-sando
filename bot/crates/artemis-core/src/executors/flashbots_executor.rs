//! Phase-2 stub of the Flashbots executor.
//!
//! The legacy implementation depended on `ethers-flashbots`, which has no
//! alloy v1 equivalent yet. Phase 5 will replace this stub with a hand-rolled
//! relay client (raw HTTPS POST to `relay.flashbots.net` with an
//! `X-Flashbots-Signature` header signed by an `alloy_signer` wallet).
//!
//! In the meantime we keep the public surface (`FlashbotsBundle`,
//! `FlashbotsExecutor`) so downstream crates compile and wire correctly; the
//! `execute` method just logs that a bundle was received and returns `Ok(())`.

use crate::types::Executor;
use alloy::primitives::Bytes;
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;
use tracing::info;

/// A single Flashbots bundle request: raw signed transactions plus the target
/// block. Mirrors the JSON-RPC `eth_sendBundle` payload that Phase 5 will POST
/// to the relay.
#[derive(Debug, Clone)]
pub struct FlashbotsBundleRequest {
    pub txs: Vec<Bytes>,
    pub target_block: u64,
}

/// A bundle of Flashbots bundle requests. The artemis engine emits one of
/// these per strategy action; we keep the `Vec` shape because the original
/// strategy may produce multiple bundles per event.
pub type FlashbotsBundle = Vec<FlashbotsBundleRequest>;

/// Phase-2 stub. Holds the relay URL and signer marker only; does not perform
/// any network IO.
pub struct FlashbotsExecutor<S> {
    relay_url: String,
    _signer: PhantomData<S>,
}

impl<S> FlashbotsExecutor<S> {
    pub fn new(relay_url: impl Into<String>) -> Self {
        Self {
            relay_url: relay_url.into(),
            _signer: PhantomData,
        }
    }
}

#[async_trait]
impl<S> Executor<FlashbotsBundle> for FlashbotsExecutor<S>
where
    S: Send + Sync + 'static,
{
    async fn execute(&self, action: FlashbotsBundle) -> Result<()> {
        info!(
            relay = %self.relay_url,
            bundles = action.len(),
            "FlashbotsExecutor stub: bundle send is deferred to Phase 5"
        );
        Ok(())
    }
}
