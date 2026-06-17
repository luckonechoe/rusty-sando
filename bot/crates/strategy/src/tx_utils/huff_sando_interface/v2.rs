use alloy::primitives::{Address, U256};

use super::common::{
    five_byte_encoder::FiveByteMetaData,
    get_jump_dest_from_sig,
    packed_encoder::{abi::encode_packed, SolidityDataType, TakeLastXBytes},
    weth_encoder::WethEncoder,
    WETH_ADDRESS,
};

/// Minimal pool view consumed by the V2 huff encoders.
///
/// Phase 3a replaces the `cfmms::pool::UniswapV2Pool` input (cfmms is removed in
/// the alloy migration) with this self-contained struct. The huff frontrun /
/// backrun encoders only read `address`; `token_a` / `token_b` are carried so
/// the sim-only `lil_router_interface` (which must pick the non-WETH token) can
/// share the same pool type, mirroring the original cfmms struct shape.
#[derive(Debug, Clone, Copy)]
pub struct UniswapV2Pool {
    pub address: Address,
    pub token_a: Address,
    pub token_b: Address,
}

impl UniswapV2Pool {
    pub fn address(&self) -> Address {
        self.address
    }
}

pub fn v2_create_frontrun_payload(
    pool: UniswapV2Pool,
    output_token: Address,
    amount_in: U256,
    amount_out: U256, // amount_out is needed to be passed due to taxed tokens
) -> (Vec<u8>, U256) {
    let jump_dest = get_jump_dest_from_sig(if WETH_ADDRESS < output_token {
        "v2_frontrun0"
    } else {
        "v2_frontrun1"
    });

    let five_bytes =
        FiveByteMetaData::encode(amount_out, if WETH_ADDRESS < output_token { 1 } else { 0 });

    let (payload, _) = encode_packed(&[
        SolidityDataType::NumberWithShift(U256::from(jump_dest), TakeLastXBytes(8)),
        SolidityDataType::Address(pool.address()),
        SolidityDataType::Bytes(&five_bytes.finalize_to_bytes()),
    ]);

    let encoded_call_value = WethEncoder::encode(amount_in);

    (payload, encoded_call_value)
}

/// dev: amount_out is needed to be passed due to taxed tokens
pub fn v2_create_backrun_payload(
    pool: UniswapV2Pool,
    input_token: Address,
    amount_in: U256,
    amount_out: U256, // amount_out is needed to be passed due to taxed tokens
) -> (Vec<u8>, U256) {
    let jump_dest = get_jump_dest_from_sig(if WETH_ADDRESS < input_token {
        "v2_backrun0"
    } else {
        "v2_backrun1"
    });

    let five_bytes = FiveByteMetaData::encode(amount_in, 1);

    let (payload, _) = encode_packed(&[
        SolidityDataType::NumberWithShift(U256::from(jump_dest), TakeLastXBytes(8)),
        SolidityDataType::Address(pool.address()),
        SolidityDataType::Address(input_token),
        SolidityDataType::Bytes(&five_bytes.finalize_to_bytes()),
    ]);

    let encoded_call_value = WethEncoder::encode(amount_out);

    (payload, encoded_call_value)
}
