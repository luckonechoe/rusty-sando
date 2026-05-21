//! Sandwich bot entrypoint.
//!
//! Phase 2 wires up only the alloy v1 provider, the block collector,
//! the mempool collector (with the EIP-4844 blob-tx filter), and the
//! Phase-2 stub `FlashbotsExecutor`. The real sandwich strategy
//! (`strategy::bot::SandoBot`) is gated behind the `simulate` feature
//! and ships in Phases 3–5 along with the revm simulator, CFMM math,
//! and the hand-rolled Flashbots relay client.

use std::sync::Arc;

use alloy::providers::ProviderBuilder;
use anyhow::Result;
use artemis_core::{
    collectors::{block_collector::BlockCollector, mempool_collector::MempoolCollector},
    engine::Engine,
    executors::flashbots_executor::{FlashbotsBundle, FlashbotsExecutor},
    types::{CollectorMap, ExecutorMap},
};
use log::info;
use rusty_sando::{
    config::Config,
    initialization::{print_banner, setup_logger},
};

/// Engine-level event enum. Phase 5 will extend this with strategy-specific
/// variants (mempool tx, pool sync, etc.).
#[derive(Debug, Clone)]
pub enum Event {
    NewBlock(artemis_core::collectors::block_collector::NewBlock),
    NewTransaction(alloy::rpc::types::Transaction),
}

/// Engine-level action enum. Phase 5 will replace this with the real
/// `Action::SubmitToFlashbots(FlashbotsBundle)` variant.
#[derive(Debug, Clone)]
pub enum Action {
    SubmitToFlashbots(FlashbotsBundle),
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    print_banner();
    let config = Config::read_from_dotenv().await?;

    // Alloy v1 pub-sub provider over WSS.
    //
    // NOTE(alloy v1): `ProviderBuilder::new().connect(<wss-url>)` auto-detects
    // the WebSocket transport from the URL scheme (ws:// or wss://). The
    // legacy 0.x `WsConnect`/`on_ws` builder helpers were removed in alloy
    // v1; using `connect(...)` keeps us on the v1-stable API surface.
    let provider = Arc::new(
        ProviderBuilder::new()
            .connect(config.wss_rpc.as_str())
            .await?,
    );

    // Engine.
    let mut engine: Engine<Event, Action> = Engine::default();

    // Block collector.
    let block_collector = Box::new(BlockCollector::new(provider.clone()));
    let block_collector = CollectorMap::new(block_collector, Event::NewBlock);
    engine.add_collector(Box::new(block_collector));

    // Mempool collector (EIP-4844 blob carriers filtered out at source).
    let mempool_collector = Box::new(MempoolCollector::new(provider.clone()));
    let mempool_collector = CollectorMap::new(mempool_collector, Event::NewTransaction);
    engine.add_collector(Box::new(mempool_collector));

    // Strategy: gated behind the `simulate` feature in the `strategy` crate;
    // Phase 5 plugs `SandoBot::new(provider.clone(), cfg)` in here.
    let _ = (
        config.searcher_signer,
        config.bundle_signer,
        config.sando_address,
        config.sando_inception_block,
        config.discord_webhook,
    );

    // Flashbots executor stub.
    let executor = Box::new(FlashbotsExecutor::<alloy::signers::local::PrivateKeySigner>::new(
        "https://relay.flashbots.net",
    ));
    let executor = ExecutorMap::new(executor, |action| match action {
        Action::SubmitToFlashbots(bundle) => Some(bundle),
    });
    engine.add_executor(Box::new(executor));

    // Run.
    if let Ok(mut set) = engine.run().await {
        while let Some(res) = set.join_next().await {
            info!("res: {:?}", res)
        }
    }

    Ok(())
}
