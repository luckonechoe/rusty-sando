use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::Transaction;
use anyhow::Result;
use dashmap::DashMap;
use std::{str::FromStr, sync::Arc};

use crate::types::Pool;

/// Supported DEX variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexVariant {
    UniswapV2,
    UniswapV3,
}

/// Static factory-address + variant + inception-block descriptor.
#[derive(Debug, Clone)]
pub struct DexDescriptor {
    pub factory: Address,
    pub variant: DexVariant,
    pub inception_block: u64,
}

pub(crate) struct PoolManager<P> {
    /// Provider (Phase 4: used for pool sync)
    provider: Arc<P>,
    /// Sandwichable pools
    pools: DashMap<Address, Pool>,
    /// Which DEXes to monitor
    dexes: Vec<DexDescriptor>,
}

impl<P: Provider + 'static> PoolManager<P> {
    /// Sync pool state. Phase 4 placeholder.
    pub async fn setup(&mut self) -> Result<()> {
        todo!("Phase 4: CFMM/state-diff")
    }

    /// Return a tx's touched sandwichable pools. Phase 4 placeholder.
    pub async fn get_touched_sandwichable_pools(
        &self,
        _victim_tx: &Transaction,
        _provider: Arc<P>,
    ) -> Result<Vec<Pool>> {
        todo!("Phase 4: CFMM/state-diff")
    }

    pub fn new(provider: Arc<P>) -> Self {
        // Static factory list — pure address data, no legacy deps.
        let dexes_data: &[(&str, DexVariant, u64)] = &[
            (
                "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f", // Uniswap V2
                DexVariant::UniswapV2,
                10_000_835,
            ),
            (
                "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac", // SushiSwap
                DexVariant::UniswapV2,
                10_794_229,
            ),
            (
                "0x9DEB29c9a4c7A88a3C0257393b7f3335338D9A9D", // Crypto.com swap
                DexVariant::UniswapV2,
                10_828_414,
            ),
            (
                "0x4eef5746ED22A2fD368629C1852365bf5dcb79f1", // Convergence
                DexVariant::UniswapV2,
                12_385_067,
            ),
            (
                "0x1097053Fd2ea711dad45caCcc45EfF7548fCB362", // PancakeSwap
                DexVariant::UniswapV2,
                15_614_590,
            ),
            (
                "0x115934131916C8b277DD010Ee02de363c09d037c", // ShibaSwap
                DexVariant::UniswapV2,
                12_771_526,
            ),
            (
                "0x35113a300ca0D7621374890ABFEAC30E88f214b1", // SaitaSwap
                DexVariant::UniswapV2,
                15_210_780,
            ),
            (
                "0x1F98431c8aD98523631AE4a59f267346ea31F984", // Uniswap V3
                DexVariant::UniswapV3,
                12_369_621,
            ),
        ];

        let dexes = dexes_data
            .iter()
            .map(|(addr, variant, block)| DexDescriptor {
                factory: Address::from_str(addr).unwrap(),
                variant: *variant,
                inception_block: *block,
            })
            .collect();

        Self {
            pools: DashMap::new(),
            provider,
            dexes,
        }
    }
}
