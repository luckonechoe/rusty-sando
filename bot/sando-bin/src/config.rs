//! Runtime configuration for the sandwich bot.
//!
//! Phase 2 ports the legacy ethers-based config to alloy v1:
//! - `LocalWallet`            -> `alloy::signers::local::PrivateKeySigner`
//! - `ethers::types::Address` -> `alloy::primitives::Address`
//! - `ethers::types::U64`     -> plain `u64` (block numbers fit fine)
//!
//! The `Url` for the upstream WSS RPC is kept on `reqwest::Url` for
//! consistency with the rest of the workspace.

use std::{env, str::FromStr};

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Result};
use dotenv::dotenv;
use reqwest::Url;

pub struct Config {
    pub searcher_signer: PrivateKeySigner,
    pub sando_inception_block: u64,
    pub sando_address: Address,
    pub bundle_signer: PrivateKeySigner,
    pub wss_rpc: Url,
    pub discord_webhook: String,
}

impl Config {
    pub async fn read_from_dotenv() -> Result<Self> {
        dotenv().ok();

        let get_env = |var: &str| {
            env::var(var).map_err(|_| anyhow!("Required environment variable \"{}\" not set", var))
        };

        let searcher_signer = get_env("SEARCHER_PRIVATE_KEY")?
            .parse::<PrivateKeySigner>()
            .map_err(|_| anyhow!("Failed to parse \"SEARCHER_PRIVATE_KEY\""))?;

        let sando_inception_block = get_env("SANDWICH_INCEPTION_BLOCK")?
            .parse::<u64>()
            .map_err(|_| anyhow!("Failed to parse \"SANDWICH_INCEPTION_BLOCK\" into u64"))?;

        let sando_address = Address::from_str(&get_env("SANDWICH_CONTRACT")?)
            .map_err(|_| anyhow!("Failed to parse \"SANDWICH_CONTRACT\""))?;

        let bundle_signer = get_env("FLASHBOTS_AUTH_KEY")?
            .parse::<PrivateKeySigner>()
            .map_err(|_| anyhow!("Failed to parse \"FLASHBOTS_AUTH_KEY\""))?;

        let wss_rpc = get_env("WSS_RPC")?
            .parse()
            .map_err(|_| anyhow!("Failed to parse \"WSS_RPC\""))?;

        let discord_webhook = get_env("DISCORD_WEBHOOK")?;

        Ok(Self {
            searcher_signer,
            sando_inception_block,
            sando_address,
            bundle_signer,
            wss_rpc,
            discord_webhook,
        })
    }
}
