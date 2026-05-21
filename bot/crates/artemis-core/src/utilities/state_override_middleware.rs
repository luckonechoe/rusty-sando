//! Phase-2 stub of the state-override middleware.
//!
//! The legacy ethers `Middleware` tower pattern does not exist in alloy v1:
//! state overrides are a per-call argument on `eth_call` (and `debug_traceCall`)
//! rather than a wrapping middleware. Phase 3 (simulator) will use
//! `alloy::rpc::types::state::StateOverride` directly inside revm fork
//! drivers, so this module exists only to keep the historical public surface
//! compiling.
//!
//! Downstream code that used to invoke `middleware.call(tx, block).await` now
//! passes the `StateOverride` directly via `Provider::call(&tx).overrides(&state)`.

use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::rpc::types::state::StateOverride;

/// Lightweight container that holds a provider handle and a state-override
/// map. Replaces the ethers `Middleware`-based version one-for-one in the
/// `set_code` / `add_code_to_address` API.
#[derive(Debug, Clone)]
pub struct StateOverrideMiddleware<P> {
    inner: P,
    state: StateOverride,
}

impl<P> StateOverrideMiddleware<P> {
    /// Create a new state-override container around the given provider.
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            state: StateOverride::default(),
        }
    }

    /// Borrow the inner provider.
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Borrow the underlying state-override map. Pass this to
    /// `Provider::call(...).overrides(&map)` at the call site.
    pub fn state(&self) -> &StateOverride {
        &self.state
    }

    /// Override the bytecode at `address`.
    pub fn add_code_to_address(&mut self, address: Address, code: Bytes) {
        self.state.entry(address).or_default().code = Some(code);
    }

    /// Set a single storage slot at `address`.
    pub fn set_storage(&mut self, address: Address, slot: B256, value: B256) {
        let acct = self.state.entry(address).or_default();
        let storage = acct.state_diff.get_or_insert_with(Default::default);
        storage.insert(slot, value);
    }

    /// Override the balance of an address.
    pub fn set_balance(&mut self, address: Address, balance: U256) {
        self.state.entry(address).or_default().balance = Some(balance);
    }
}
