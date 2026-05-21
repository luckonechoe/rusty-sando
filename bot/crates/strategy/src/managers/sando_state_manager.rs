use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use std::sync::Arc;

pub struct SandoStateManager {
    sando_contract: Address,
    sando_inception_block: u64,
    searcher_signer: PrivateKeySigner,
    weth_inventory: U256,
    token_dust: Vec<Address>,
}

impl SandoStateManager {
    pub fn new(
        sando_contract: Address,
        searcher_signer: PrivateKeySigner,
        sando_inception_block: u64,
    ) -> Self {
        Self {
            sando_contract,
            sando_inception_block,
            searcher_signer,
            weth_inventory: U256::ZERO,
            token_dust: Vec::new(),
        }
    }

    /// Sync WETH inventory and token dust via provider.
    ///
    /// Phase 3 placeholder — alloy provider + ERC20 calls deferred.
    pub async fn setup<P: Provider + 'static>(&mut self, _provider: Arc<P>) -> Result<()> {
        todo!("Phase 3: alloy provider + ERC20 calls")
    }

    pub fn get_sando_address(&self) -> Address {
        self.sando_contract
    }

    pub fn get_searcher_address(&self) -> Address {
        self.searcher_signer.address()
    }

    pub fn get_searcher_signer(&self) -> &PrivateKeySigner {
        &self.searcher_signer
    }

    pub fn get_weth_inventory(&self) -> U256 {
        self.weth_inventory
    }
}
