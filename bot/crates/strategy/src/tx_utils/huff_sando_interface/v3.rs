use alloy::primitives::{keccak256, Address, U256};
use alloy::sol_types::SolValue;

use super::common::{
    five_byte_encoder::FiveByteMetaData,
    get_jump_dest_from_sig,
    packed_encoder::{abi::encode_packed, SolidityDataType, TakeLastXBytes},
    weth_encoder::WethEncoder,
    WETH_ADDRESS,
};

/// Minimal pool view consumed by the V3 huff encoders.
///
/// Phase 3a replaces the `cfmms::pool::UniswapV3Pool` input (cfmms is removed in
/// the alloy migration) with this self-contained struct that carries only the
/// fields the encoder actually reads: `address`, `token_a`, `token_b`, `fee`.
#[derive(Debug, Clone, Copy)]
pub struct UniswapV3Pool {
    pub address: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub fee: u32,
}

impl UniswapV3Pool {
    pub fn address(&self) -> Address {
        self.address
    }
}

pub fn v3_create_frontrun_payload(
    pool: UniswapV3Pool,
    output_token: Address,
    amount_in: U256,
) -> (Vec<u8>, U256) {
    let (payload, _) = encode_packed(&[
        SolidityDataType::NumberWithShift(
            U256::from(get_jump_dest_from_sig(if WETH_ADDRESS < output_token {
                "v3_frontrun0"
            } else {
                "v3_frontrun1"
            })),
            TakeLastXBytes(8),
        ),
        SolidityDataType::Address(pool.address()),
        SolidityDataType::Bytes(&get_pool_key_hash(pool).to_vec()),
    ]);

    let encoded_value = WethEncoder::encode(amount_in);

    (payload, encoded_value)
}

pub fn v3_create_backrun_payload(
    pool: UniswapV3Pool,
    input_token: Address,
    amount_in: U256,
) -> Vec<u8> {
    let five_bytes = FiveByteMetaData::encode(amount_in, 2);

    let (payload, _) = encode_packed(&[
        SolidityDataType::NumberWithShift(
            U256::from(get_jump_dest_from_sig(if WETH_ADDRESS < input_token {
                "v3_backrun0"
            } else {
                "v3_backrun1"
            })),
            TakeLastXBytes(8),
        ),
        SolidityDataType::Address(pool.address()),
        SolidityDataType::Address(input_token),
        SolidityDataType::Bytes(&get_pool_key_hash(pool).to_vec()),
        SolidityDataType::Bytes(&five_bytes.finalize_to_bytes()),
    ]);

    payload
}

/// https://github.com/Uniswap/v3-periphery/blob/6cce88e63e176af1ddb6cc56e029110289622317/contracts/libraries/PoolAddress.sol#L41C80-L41C80
fn get_pool_key_hash(pool: UniswapV3Pool) -> [u8; 32] {
    // Standard (32-byte-padded) ABI encoding of (address, address, uint24-as-uint256),
    // matching the original `ethers::abi::encode([Address, Address, Uint])`.
    let encoded = (pool.token_a, pool.token_b, U256::from(pool.fee)).abi_encode();
    keccak256(encoded).0
}
