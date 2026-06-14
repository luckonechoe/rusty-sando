//! Phase 3a golden-hex fixtures for the alloy-migrated tx_utils encoders.
//!
//! Sourcing of golden vectors
//! --------------------------
//! * `abi.encodePacked` vectors (`packed_encoder` tests) are taken verbatim from
//!   the upstream `eth_encode_packed` crate's own published reference tests
//!   (docs.rs `eth-encode-packed` v0.1.0 `src/lib.rs`), which is the library the
//!   legacy encoders used. Matching them proves the local alloy-native shim is
//!   byte-for-byte equivalent to the crate it replaces.
//! * Uniswap V3 pool-key hashes and the full v2/v3 huff payloads were computed
//!   independently with a from-scratch Keccak-256 implementation in Python
//!   (validated against the canonical `keccak256("") = c5d2...a470` KAT), then
//!   frozen here as literals.
//! * `lil_router` calldata selectors/encodings were likewise computed offline
//!   from the function signatures (`keccak256(sig)[..4]` + standard 32-byte ABI
//!   word encoding).

use alloy::primitives::{address, hex, Address, U256};

use crate::tx_utils::huff_sando_interface::common::five_byte_encoder::FiveByteMetaData;
use crate::tx_utils::huff_sando_interface::common::packed_encoder::{
    abi::encode_packed, SolidityDataType, TakeLastXBytes,
};
use crate::tx_utils::huff_sando_interface::common::weth_encoder::WethEncoder;
use crate::tx_utils::huff_sando_interface::common::WETH_ADDRESS;
use crate::tx_utils::huff_sando_interface::v2::{
    v2_create_backrun_payload, v2_create_frontrun_payload, UniswapV2Pool,
};
use crate::tx_utils::huff_sando_interface::v3::{
    v3_create_backrun_payload, v3_create_frontrun_payload, UniswapV3Pool,
};

const POOL: Address = address!("1111111111111111111111111111111111111111");
const USDC: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
// An arbitrary token whose integer value is greater than WETH's (for branch
// selection coverage).
const TOKEN_GT_WETH: Address = address!("D02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");

// ---------------------------------------------------------------------------
// packed_encoder: byte-equivalence vs. eth_encode_packed reference vectors
// ---------------------------------------------------------------------------

#[test]
fn packed_uint24_take_last_3_bytes() {
    // eth_encode_packed reference: uint24(0xfa1) => "000fa1"
    let (bytes, s) = encode_packed(&[SolidityDataType::NumberWithShift(
        U256::from(0xfa1),
        TakeLastXBytes(24),
    )]);
    assert_eq!(s, "000fa1");
    assert_eq!(hex::encode(&bytes), "000fa1");
}

#[test]
fn packed_uint8_and_uint32() {
    // single byte
    let (b1, _) = encode_packed(&[SolidityDataType::NumberWithShift(
        U256::from(0x2a),
        TakeLastXBytes(8),
    )]);
    assert_eq!(hex::encode(b1), "2a");
    // four bytes
    let (b2, _) = encode_packed(&[SolidityDataType::NumberWithShift(
        U256::from(0xdeadbeefu64),
        TakeLastXBytes(32),
    )]);
    assert_eq!(hex::encode(b2), "deadbeef");
}

#[test]
fn packed_address_is_20_bytes() {
    let (bytes, s) = encode_packed(&[SolidityDataType::Address(WETH_ADDRESS)]);
    assert_eq!(s, "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    assert_eq!(bytes.len(), 20);
}

#[test]
fn packed_mixed_concatenation() {
    let raw = [0xaa, 0xbb];
    let (bytes, _) = encode_packed(&[
        SolidityDataType::NumberWithShift(U256::from(0x05), TakeLastXBytes(8)),
        SolidityDataType::Address(POOL),
        SolidityDataType::Bytes(&raw),
    ]);
    assert_eq!(
        hex::encode(bytes),
        "051111111111111111111111111111111111111111aabb"
    );
}

// ---------------------------------------------------------------------------
// weth_encoder
// ---------------------------------------------------------------------------

#[test]
fn weth_encode_decode() {
    assert_eq!(
        WethEncoder::encode(U256::from(1_000_000_000_000_000_000u64)),
        U256::from(10_000_000_000_000u64)
    );
    assert_eq!(
        WethEncoder::decode(U256::from(7u64)),
        U256::from(700_000u64)
    );
}

// ---------------------------------------------------------------------------
// five_byte_encoder
// ---------------------------------------------------------------------------

#[test]
fn five_byte_finalize_param1() {
    let fb = FiveByteMetaData::encode(U256::from(12_345_678_901_234_567_890u128), 1);
    assert_eq!(hex::encode(fb.finalize_to_bytes()), "3cab54a98c");
}

#[test]
fn five_byte_finalize_param2() {
    let fb = FiveByteMetaData::encode(U256::from(1_000_000_000_000_000_000u64), 2);
    assert_eq!(hex::encode(fb.finalize_to_bytes()), "5c0de0b6b3");
}

#[test]
fn five_byte_decode_roundtrip_is_lossy_but_stable() {
    let fb = FiveByteMetaData::encode(U256::from(1_000_000_000_000_000_000u64), 2);
    // 232830643 << 32
    assert_eq!(fb.decode(), U256::from(232_830_643u64) << 32);
}

// ---------------------------------------------------------------------------
// v2 payloads
// ---------------------------------------------------------------------------

#[test]
fn v2_frontrun_branch0_when_weth_lt_output() {
    // WETH < TOKEN_GT_WETH => "v2_frontrun0" => jump dest 0x0a
    let pool = UniswapV2Pool {
        address: POOL,
        token_a: USDC,
        token_b: WETH_ADDRESS,
    };
    let (payload, value) = v2_create_frontrun_payload(
        pool,
        TOKEN_GT_WETH,
        U256::from(1_000_000_000_000_000_000u64),
        U256::from(12_345_678_901_234_567_890u128),
    );
    assert_eq!(
        hex::encode(payload),
        "0a11111111111111111111111111111111111111113cab54a98c"
    );
    assert_eq!(value, U256::from(10_000_000_000_000u64));
}

#[test]
fn v2_frontrun_branch1_when_weth_gt_output() {
    // WETH > USDC => "v2_frontrun1" => jump dest 0x14, param_index 0
    let pool = UniswapV2Pool {
        address: POOL,
        token_a: USDC,
        token_b: WETH_ADDRESS,
    };
    let (payload, _) = v2_create_frontrun_payload(
        pool,
        USDC,
        U256::from(1_000_000_000_000_000_000u64),
        U256::from(12_345_678_901_234_567_890u128),
    );
    assert_eq!(
        hex::encode(payload),
        "1411111111111111111111111111111111111111111cab54a98c"
    );
}

#[test]
fn v2_backrun_branch1_when_weth_gt_input() {
    // WETH > USDC => "v2_backrun1" => jump dest 0x0f
    let pool = UniswapV2Pool {
        address: POOL,
        token_a: USDC,
        token_b: WETH_ADDRESS,
    };
    let (payload, value) = v2_create_backrun_payload(
        pool,
        USDC,
        U256::from(1_000_000_000_000_000_000u64),
        U256::from(12_345_678_901_234_567_890u128),
    );
    assert_eq!(
        hex::encode(payload),
        "0f1111111111111111111111111111111111111111a0b86991c6218b36c1d19d4a2e9eb0ce3606eb483c0de0b6b3"
    );
    assert_eq!(value, U256::from(123_456_789_012_345u64));
}

// ---------------------------------------------------------------------------
// v3 payloads + pool key hash
// ---------------------------------------------------------------------------

const USDC_WETH_500_KEYHASH: &str =
    "08374668a423750b443f65d645c5693995d43722b42cd84f7eeba28b008a40a2";

#[test]
fn v3_frontrun_branch1_with_keyhash() {
    // pool token_a=USDC token_b=WETH fee=500; output USDC < WETH => "v3_frontrun1" => 0x28
    let pool = UniswapV3Pool {
        address: POOL,
        token_a: USDC,
        token_b: WETH_ADDRESS,
        fee: 500,
    };
    let (payload, value) =
        v3_create_frontrun_payload(pool, USDC, U256::from(1_000_000_000_000_000_000u64));
    let expected = format!(
        "281111111111111111111111111111111111111111{}",
        USDC_WETH_500_KEYHASH
    );
    assert_eq!(hex::encode(payload), expected);
    assert_eq!(value, U256::from(10_000_000_000_000u64));
}

#[test]
fn v3_backrun_branch1_with_keyhash_and_five_bytes() {
    let pool = UniswapV3Pool {
        address: POOL,
        token_a: USDC,
        token_b: WETH_ADDRESS,
        fee: 500,
    };
    let payload =
        v3_create_backrun_payload(pool, USDC, U256::from(1_000_000_000_000_000_000u64));
    let expected = format!(
        "231111111111111111111111111111111111111111a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48{}5c0de0b6b3",
        USDC_WETH_500_KEYHASH
    );
    assert_eq!(hex::encode(payload), expected);
}

// ---------------------------------------------------------------------------
// lil_router_interface (sim-only)
// ---------------------------------------------------------------------------

#[cfg(feature = "simulate")]
mod lil_router {
    use super::*;
    use alloy::primitives::I256;

    use crate::tx_utils::lil_router_interface::{build_swap_v2_data, build_swap_v3_data};

    #[test]
    fn swap_v2_calldata_frontrun() {
        // pool tokens {USDC, WETH}; frontrun => WETH(in) -> USDC(out)
        // selector 81eeb93c + amountIn + targetPair(pool) + inputToken(WETH) + outputToken(USDC)
        let pool = UniswapV2Pool {
            address: POOL,
            token_a: USDC,
            token_b: WETH_ADDRESS,
        };
        let data = build_swap_v2_data(U256::from(1_000_000_000_000_000_000u64), pool, true);
        let expected = "81eeb93c0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000001111111111111111111111111111111111111111000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        assert_eq!(hex::encode(data), expected);
    }

    #[test]
    fn swap_v3_calldata_frontrun() {
        let pool = UniswapV3Pool {
            address: POOL,
            token_a: USDC,
            token_b: WETH_ADDRESS,
            fee: 500,
        };
        let data = build_swap_v3_data(
            I256::try_from(1_000_000_000_000_000_000i64).unwrap(),
            pool,
            true,
        );
        let expected = "4b588d400000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000001111111111111111111111111111111111111111000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        assert_eq!(hex::encode(data), expected);
    }
}
