//! Strategy crate.
//!
//! Phase 2 (provider/types layer migration to alloy) deliberately leaves the
//! sandwich strategy itself behind a `simulate` cargo feature. Every module
//! in this crate currently depends on the legacy `ethers` + `foundry-evm`
//! simulator stack, the `abigen!` ABI bindings, and the Huff sando contract
//! interface — all of which belong to Phase 3 (revm simulator), Phase 4
//! (CFMM math), and Phase 5 (bundle/relay). They will be ported in those
//! phases.
//!
//! Until then this crate compiles to an empty surface so the workspace stays
//! green for `cargo check` against `artemis-core`.

#![cfg_attr(not(feature = "simulate"), allow(dead_code))]

#[cfg(feature = "simulate")]
mod abi;
#[cfg(feature = "simulate")]
mod constants;
#[cfg(feature = "simulate")]
mod helpers;
#[cfg(feature = "simulate")]
mod simulator;

#[cfg(feature = "simulate")]
mod managers;

#[cfg(feature = "simulate")]
mod tx_utils;

#[cfg(feature = "simulate")]
pub mod bot;

#[cfg(feature = "simulate")]
pub mod types;
