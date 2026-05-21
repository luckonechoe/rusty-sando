//! Strategy crate — Phase 2b.
//!
//! Non-simulator modules (`abi`, `constants`, `helpers`, `managers`, `types`)
//! are now ungated and compile against alloy v1.  Bodies that depend on the
//! revm simulator, CFMM math, or Flashbots bundle building carry `todo!()`
//! markers and will be filled in Phase 3 / 4 / 5.
//!
//! The following modules remain behind the `simulate` cargo feature because
//! they still depend on the legacy revm/foundry/Huff stack that has not yet
//! been ported:
//!   - `simulator`   (Phase 3: revm executor)
//!   - `tx_utils`    (Phase 5: Huff sando interface)
//!   - `bot`         (Phase 5: strategy orchestration)

#![allow(dead_code)]

pub mod abi;
pub mod constants;
pub mod helpers;
pub mod managers;
pub mod types;

#[cfg(feature = "simulate")]
mod simulator;

#[cfg(feature = "simulate")]
mod tx_utils;

#[cfg(feature = "simulate")]
pub mod bot;
