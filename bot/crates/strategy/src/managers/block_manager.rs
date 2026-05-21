use alloy::providers::Provider;
use anyhow::Result;
use std::sync::Arc;

use crate::types::BlockInfo;

pub struct BlockManager {
    latest_block: BlockInfo,
    next_block: BlockInfo,
}

impl BlockManager {
    pub fn new() -> Self {
        Self {
            latest_block: BlockInfo::default(),
            next_block: BlockInfo::default(),
        }
    }

    /// Sync to the latest on-chain block via the alloy provider.
    ///
    /// Phase 3 placeholder — provider integration is deferred.
    pub async fn setup<P: Provider + 'static>(&mut self, _provider: Arc<P>) -> Result<()> {
        todo!("Phase 3: provider integration")
    }

    /// Return info for the next block.
    pub fn get_next_block(&self) -> BlockInfo {
        self.next_block
    }

    /// Return info for the latest mined block.
    pub fn get_latest_block(&self) -> BlockInfo {
        self.latest_block
    }

    /// Update internal state with the latest mined block and compute next block.
    pub fn update_block_info<T: Into<BlockInfo>>(&mut self, latest_block: T) {
        let latest_block: BlockInfo = latest_block.into();
        self.latest_block = latest_block;
        self.next_block = latest_block.get_next_block();
    }
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new()
    }
}
