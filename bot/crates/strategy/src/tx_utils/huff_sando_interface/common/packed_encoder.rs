//! Alloy-native `abi.encodePacked` shim.
//!
//! Phase 3a migration note
//! -----------------------
//! The legacy encoders relied on the `eth_encode_packed` crate
//! (`SolidityDataType` / `TakeLastXBytes`). That crate is built on the
//! `ethabi` / `ethereum-types` (ethers-era) type stack, so re-adding it to an
//! alloy-v1-only build would drag non-alloy primitive types back into the
//! dependency graph. Instead we reimplement the exact same packed-encoding
//! semantics here on top of `alloy_primitives`. The public surface
//! (`SolidityDataType`, `TakeLastXBytes`, `abi::encode_packed`) mirrors the
//! original crate one-to-one so the call sites stay byte-for-byte identical.
//!
//! Byte-equivalence against the original `eth_encode_packed` reference vectors
//! is locked in by the golden-hex tests in `tx_utils/tests.rs`.

use alloy::primitives::{Address, U256};

/// Number of trailing bits to keep when packing a `NumberWithShift` value.
///
/// Mirrors `eth_encode_packed::TakeLastXBytes`. Despite the name, the original
/// crate's argument is expressed in **bits** (e.g. `TakeLastXBytes(8)` keeps the
/// last byte, `TakeLastXBytes(32)` keeps the last 4 bytes).
pub struct TakeLastXBytes(pub usize);

/// Subset of the original `eth_encode_packed::SolidityDataType` used by the
/// huff sando encoders. Each variant packs with no padding, exactly like
/// Solidity's `abi.encodePacked`.
pub enum SolidityDataType<'a> {
    /// A `uintN` value; only the last `TakeLastXBytes(bits)` are emitted,
    /// taken from the big-endian 32-byte representation.
    NumberWithShift(U256, TakeLastXBytes),
    /// A 20-byte address.
    Address(Address),
    /// Raw, already-encoded bytes, emitted verbatim.
    Bytes(&'a [u8]),
}

/// Reimplementation of `eth_encode_packed::abi`.
pub mod abi {
    use super::{SolidityDataType, TakeLastXBytes};

    /// Packs the given items with `abi.encodePacked` semantics.
    ///
    /// Returns `(bytes, hex_string)` to mirror the original crate's signature;
    /// the encoders only consume the first element.
    pub fn encode_packed(items: &[SolidityDataType]) -> (Vec<u8>, String) {
        let mut out: Vec<u8> = Vec::new();
        for item in items {
            encode_one(item, &mut out);
        }
        let hex_str = alloy::primitives::hex::encode(&out);
        (out, hex_str)
    }

    fn encode_one(item: &SolidityDataType, out: &mut Vec<u8>) {
        match item {
            SolidityDataType::Address(a) => {
                out.extend_from_slice(a.as_slice());
            }
            SolidityDataType::Bytes(b) => {
                out.extend_from_slice(b);
            }
            SolidityDataType::NumberWithShift(n, TakeLastXBytes(bits)) => {
                // Original crate renders the value as a 32-byte big-endian word
                // and keeps the last `bits / 8` bytes.
                let full = n.to_be_bytes::<32>();
                let keep = bits / 8;
                out.extend_from_slice(&full[32 - keep..]);
            }
        }
    }
}
