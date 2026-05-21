use alloy::consensus::BlockHeader;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::Transaction;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Result};
use artemis_core::{
    collectors::block_collector::NewBlock, executors::flashbots_executor::FlashbotsBundle,
};

/// Core Event enum for current strategy
#[derive(Debug, Clone)]
pub enum Event {
    NewBlock(NewBlock),
    NewTransaction(Transaction),
}

/// Core Action enum for current strategy
#[derive(Debug, Clone)]
pub enum Action {
    SubmitToFlashbots(FlashbotsBundle),
}

/// Configuration for variables needed for sandwiches
#[derive(Debug, Clone)]
pub struct StratConfig {
    pub sando_address: Address,
    pub sando_inception_block: u64,
    pub searcher_signer: PrivateKeySigner,
}

// ---------------------------------------------------------------------------
// Pool placeholder (Phase 4: CFMM pool variants)
// ---------------------------------------------------------------------------

/// Placeholder pool type until CFMM math is ported in Phase 4.
#[derive(Debug, Clone)]
pub enum Pool {
    // TODO Phase 4: UniswapV2(UniswapV2Pool), UniswapV3(UniswapV3Pool)
    Placeholder,
}

/// Information on potential sandwichable opportunity
#[derive(Clone)]
pub struct RawIngredients {
    /// Victim tx/s to be used in sandwich
    meats: Vec<Transaction>,
    /// Which token do start and end sandwich with
    start_end_token: Address,
    /// Which token do we hold for duration of sandwich
    intermediary_token: Address,
    /// Which pool are we targeting (Phase 4: real pool type)
    target_pool: Pool,
}

impl RawIngredients {
    pub fn new(
        meats: Vec<Transaction>,
        start_end_token: Address,
        intermediary_token: Address,
        target_pool: Pool,
    ) -> Self {
        Self {
            meats,
            start_end_token,
            intermediary_token,
            target_pool,
        }
    }

    pub fn get_start_end_token(&self) -> Address {
        self.start_end_token
    }

    pub fn get_intermediary_token(&self) -> Address {
        self.intermediary_token
    }

    pub fn get_meats_ref(&self) -> &Vec<Transaction> {
        &self.meats
    }

    pub fn get_target_pool(&self) -> &Pool {
        &self.target_pool
    }

    /// Used for logging
    pub fn print_meats(&self) -> String {
        let mut s = String::new();
        s.push('[');
        for (i, x) in self.meats.iter().enumerate() {
            s.push_str(&format!("{:?}", x.inner.tx_hash()));
            if i != self.meats.len() - 1 {
                s.push(',');
            }
        }
        s.push(']');
        s
    }
}

// ---------------------------------------------------------------------------
// BlockInfo
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
pub struct BlockInfo {
    pub number: u64,
    pub base_fee_per_gas: U256,
    pub timestamp: u64,
    // Optional because we don't know these values for `next_block`
    pub gas_used: Option<u64>,
    pub gas_limit: Option<u64>,
}

impl BlockInfo {
    /// Returns block info for the next block
    pub fn get_next_block(&self) -> BlockInfo {
        BlockInfo {
            number: self.number + 1,
            base_fee_per_gas: calculate_next_block_base_fee(self),
            timestamp: self.timestamp + 12,
            gas_used: None,
            gas_limit: None,
        }
    }
}

impl From<NewBlock> for BlockInfo {
    fn from(value: NewBlock) -> Self {
        Self {
            number: value.number,
            base_fee_per_gas: value.base_fee_per_gas,
            timestamp: value.timestamp,
            gas_used: Some(value.gas_used),
            gas_limit: Some(value.gas_limit),
        }
    }
}

impl TryFrom<alloy::rpc::types::Block> for BlockInfo {
    type Error = anyhow::Error;

    fn try_from(value: alloy::rpc::types::Block) -> std::result::Result<Self, Self::Error> {
        let h = &value.header;
        Ok(BlockInfo {
            number: h.number(),
            gas_used: Some(h.gas_used()),
            gas_limit: Some(h.gas_limit()),
            base_fee_per_gas: U256::from(
                h.base_fee_per_gas()
                    .ok_or_else(|| anyhow!("could not parse base fee when setting up `block_manager`"))?,
            ),
            timestamp: h.timestamp(),
        })
    }
}

/// Calculate the next block base fee.
///
/// Based on: <https://ethereum.stackexchange.com/questions/107173/how-is-the-base-fee-per-gas-computed-for-a-new-block>
fn calculate_next_block_base_fee(block: &BlockInfo) -> U256 {
    let current_base_fee_per_gas = block.base_fee_per_gas;

    let current_gas_used = block
        .gas_used
        .expect("can't calculate base fee from unmined block \"next_block\"");

    let current_gas_target = block
        .gas_limit
        .expect("can't calculate base fee from unmined block \"next_block\"")
        / 2;

    if current_gas_used == current_gas_target {
        current_base_fee_per_gas
    } else if current_gas_used > current_gas_target {
        let gas_used_delta = U256::from(current_gas_used - current_gas_target);
        let base_fee_per_gas_delta =
            current_base_fee_per_gas * gas_used_delta / U256::from(current_gas_target) / U256::from(8u64);
        current_base_fee_per_gas + base_fee_per_gas_delta
    } else {
        let gas_used_delta = U256::from(current_gas_target - current_gas_used);
        let base_fee_per_gas_delta =
            current_base_fee_per_gas * gas_used_delta / U256::from(current_gas_target) / U256::from(8u64);
        current_base_fee_per_gas - base_fee_per_gas_delta
    }
}

// ---------------------------------------------------------------------------
// PendingTx placeholder (replaces revm TxEnv for Phase 2)
// ---------------------------------------------------------------------------

/// Phase 3 placeholder for a pending EVM transaction (replaces `revm::primitives::TxEnv`).
#[derive(Debug, Clone, Default)]
pub struct PendingTx {
    // TODO Phase 3: populate with revm TxEnv or equivalent
}

// ---------------------------------------------------------------------------
// SandoRecipe
// ---------------------------------------------------------------------------

/// All details for capturing a sando opportunity.
pub struct SandoRecipe {
    frontrun: PendingTx,
    frontrun_gas_used: u64,
    meats: Vec<Transaction>,
    backrun: PendingTx,
    backrun_gas_used: u64,
    revenue: U256,
    target_block: BlockInfo,
}

impl SandoRecipe {
    pub fn new(
        frontrun: PendingTx,
        frontrun_gas_used: u64,
        meats: Vec<Transaction>,
        backrun: PendingTx,
        backrun_gas_used: u64,
        revenue: U256,
        target_block: BlockInfo,
    ) -> Self {
        Self {
            frontrun,
            frontrun_gas_used,
            meats,
            backrun,
            backrun_gas_used,
            revenue,
            target_block,
        }
    }

    pub fn get_revenue(&self) -> U256 {
        self.revenue
    }

    /// Turn recipe into a signed bundle that can be submitted to Flashbots.
    pub async fn to_fb_bundle(
        self,
        _sando_address: Address,
        _searcher: &PrivateKeySigner,
        _has_dust: bool,
    ) -> Result<FlashbotsBundle> {
        todo!("Phase 5: bundle submission")
    }
}
