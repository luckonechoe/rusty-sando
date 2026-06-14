use alloy::primitives::{Bytes, I256, U256};
use alloy::sol;
use alloy::sol_types::SolCall;

use super::huff_sando_interface::common::WETH_ADDRESS;
use super::huff_sando_interface::v2::UniswapV2Pool;
use super::huff_sando_interface::v3::UniswapV3Pool;

sol! {
    function calculateSwapV2(
        uint amountIn,
        address targetPair,
        address inputToken,
        address outputToken
    ) external returns (uint amountOut, uint realAfterBalance);

    function calculateSwapV3(
        int amountIn,
        address targetPoolAddress,
        address inputToken,
        address outputToken
    ) external returns (uint amountOut, uint realAfterBalance);
}

// Build the data for the lil_router contract's calculateSwapV2 function
pub fn build_swap_v2_data(amount_in: U256, pool: UniswapV2Pool, is_frontrun: bool) -> Bytes {
    let other_token = [pool.token_a, pool.token_b]
        .into_iter()
        .find(|&t| t != WETH_ADDRESS)
        .unwrap();

    let (input_token, output_token) = if is_frontrun {
        // if frontrun we trade WETH -> TOKEN
        (WETH_ADDRESS, other_token)
    } else {
        // if backrun we trade TOKEN -> WETH
        (other_token, WETH_ADDRESS)
    };

    calculateSwapV2Call {
        amountIn: amount_in,
        targetPair: pool.address,
        inputToken: input_token,
        outputToken: output_token,
    }
    .abi_encode()
    .into()
}

// Build the data for the lil_router contract's calculateSwapV3 function
pub fn build_swap_v3_data(amount_in: I256, pool: UniswapV3Pool, is_frontrun: bool) -> Bytes {
    let other_token = [pool.token_a, pool.token_b]
        .into_iter()
        .find(|&t| t != WETH_ADDRESS)
        .unwrap();

    let (input_token, output_token) = if is_frontrun {
        // if frontrun we trade WETH -> TOKEN
        (WETH_ADDRESS, other_token)
    } else {
        // if backrun we trade TOKEN -> WETH
        (other_token, WETH_ADDRESS)
    };

    calculateSwapV3Call {
        amountIn: amount_in,
        targetPoolAddress: pool.address,
        inputToken: input_token,
        outputToken: output_token,
    }
    .abi_encode()
    .into()
}

// Decode the result of the lil_router contract's calculateSwapV2 function
pub fn decode_swap_v2_result(output: Bytes) -> Result<(U256, U256), alloy::sol_types::Error> {
    let ret = calculateSwapV2Call::abi_decode_returns(&output)?;
    Ok((ret.amountOut, ret.realAfterBalance))
}

// Decode the result of the lil_router contract's calculateSwapV3 function
pub fn decode_swap_v3_result(output: Bytes) -> Result<(U256, U256), alloy::sol_types::Error> {
    let ret = calculateSwapV3Call::abi_decode_returns(&output)?;
    Ok((ret.amountOut, ret.realAfterBalance))
}
