use alloy::eips::eip2930::AccessList;
use alloy::primitives::{b256, keccak256, Address, Bytes, B256, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolValue;
use anyhow::Result;

/// Sign an EIP-1559 transaction.
///
/// Phase 3 placeholder — signing requires the full revm/bundle pipeline.
pub async fn sign_eip1559(
    _tx: TransactionRequest,
    _signer_wallet: &PrivateKeySigner,
) -> Result<Bytes> {
    // TODO(phase-3): replace `todo!` body with real alloy v1 EIP-1559 signing logic.
    todo!("Phase 3: simulator/bundle signing")
}

/// Convert a revm access list to the alloy EIP-2930 `AccessList`.
///
/// Phase 3 placeholder — revm interop is deferred.
pub fn access_list_to_alloy(
    _access_list: Vec<(Address, Vec<U256>)>,
) -> AccessList {
    // TODO(phase-3): replace `todo!` body with real alloy v1 / revm interop conversion.
    todo!("Phase 3: simulator")
}

/// Convert an alloy EIP-2930 `AccessList` to the revm representation.
///
/// Phase 3 placeholder — revm interop is deferred.
pub fn access_list_to_revm(
    _access_list: AccessList,
) -> Vec<(Address, Vec<U256>)> {
    // TODO(phase-3): replace `todo!` body with real alloy v1 / revm interop conversion.
    todo!("Phase 3: simulator")
}

// ---------------------------------------------------------------------------
// Uniswap CREATE2 address derivation
// ---------------------------------------------------------------------------

/// Uniswap V2 init code hash (mainnet factories: Uniswap V2, SushiSwap).
pub const UNISWAP_V2_INIT_CODE_HASH: B256 =
    b256!("96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f");

/// Uniswap V3 init code hash.
pub const UNISWAP_V3_INIT_CODE_HASH: B256 =
    b256!("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54");

/// Derive the deterministic CREATE2 address of a Uniswap V2 pair.
pub fn derive_v2_pair(factory: Address, token_a: Address, token_b: Address) -> Address {
    let (token0, token1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };
    let mut salt_bytes = [0u8; 40];
    salt_bytes[0..20].copy_from_slice(token0.as_slice());
    salt_bytes[20..40].copy_from_slice(token1.as_slice());
    let salt = keccak256(salt_bytes);
    factory.create2(salt, UNISWAP_V2_INIT_CODE_HASH)
}

/// Derive the deterministic CREATE2 address of a Uniswap V3 pool.
pub fn derive_v3_pool(
    factory: Address,
    token_a: Address,
    token_b: Address,
    fee: u32,
) -> Address {
    let (token0, token1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };
    // salt = keccak256(abi.encode(token0, token1, fee))
    // Uses alloy's SolValue tuple encoding to correctly pad (address, address, uint24).
    // The fee is truncated to 24 bits by masking before encoding.
    let fee_u24 = fee & 0x00FF_FFFF;
    let encoded = (token0, token1, alloy::primitives::U256::from(fee_u24)).abi_encode();
    let salt = keccak256(encoded);
    factory.create2(salt, UNISWAP_V3_INIT_CODE_HASH)
}

// ---------------------------------------------------------------------------
// Logging macros
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! log_info_cyan {
    ($($arg:tt)*) => {
        log::info!("{}", format_args!($($arg)*).to_string().cyan());
    };
}

#[macro_export]
macro_rules! log_not_sandwichable {
    ($($arg:tt)*) => {
        log::info!("{}", format_args!($($arg)*).to_string().yellow())
    };
}

#[macro_export]
macro_rules! log_opportunity {
    ($meats:expr, $optimal_input:expr, $revenue:expr) => {{
        log::info!("\n{}", "[OPPORTUNITY DETECTED]".green().on_black().bold());
        log::info!(
            "{}",
            format!("meats: {}", $meats.to_string().green().on_black()).bold()
        );
        log::info!(
            "{}",
            format!(
                "optimal_input: {} wETH",
                $optimal_input.to_string().green().on_black()
            )
            .bold()
        );
        log::info!(
            "{}",
            format!(
                "revenue      : {} wETH",
                $revenue.to_string().green().on_black()
            )
            .bold()
        );
    }};
}

#[macro_export]
macro_rules! startup_info_log {
    ($($arg:tt)*) => {
        log::info!("{}", format_args!($($arg)*).to_string().on_black().yellow().bold());
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!("{}", format_args!($($arg)*).to_string().red());
    };
}

#[macro_export]
macro_rules! log_new_block_info {
    ($new_block:expr) => {
        log::info!(
            "{}",
            format!(
                "\nFound New Block\nLatest Block: (number:{:?}, timestamp:{:?}, basefee:{:?})",
                $new_block.number, $new_block.timestamp, $new_block.base_fee_per_gas,
            )
            .bright_purple()
            .on_black()
        );
    };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn derives_known_v2_pair_usdc_weth() {
        let factory = address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"); // Uniswap V2 factory
        let usdc = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        let weth = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
        let expected = address!("B4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
        assert_eq!(derive_v2_pair(factory, usdc, weth), expected);
        // commutative on token order
        assert_eq!(derive_v2_pair(factory, weth, usdc), expected);
    }

    #[test]
    fn derives_known_v3_pool_usdc_weth_30bps() {
        let factory = address!("1F98431c8aD98523631AE4a59f267346ea31F984");
        let usdc = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        let weth = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
        // Verified on Etherscan: https://etherscan.io/address/0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8
        let expected = address!("8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8");
        assert_eq!(derive_v3_pool(factory, usdc, weth, 3000), expected);
        assert_eq!(derive_v3_pool(factory, weth, usdc, 3000), expected);
    }
}
