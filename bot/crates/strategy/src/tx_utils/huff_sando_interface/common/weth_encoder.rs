use alloy::primitives::U256;

/// Constant used for encoding WETH amount.
pub const WETH_ENCODING_MULTIPLE: U256 = U256::from_limbs([100000, 0, 0, 0]);

pub struct WethEncoder {}

impl WethEncoder {
    /// Encodes a weth value to be passed to the contract through `tx.value`
    pub fn encode(value: U256) -> U256 {
        value / WETH_ENCODING_MULTIPLE
    }

    /// Decodes by multiplying amount by weth constant
    pub fn decode(value: U256) -> U256 {
        value * WETH_ENCODING_MULTIPLE
    }
}
