use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::{Keypair, read_keypair_file};
use yaml_rust::YamlLoader;

use crate::{Error, Result};

/// The schema for storage prices in accounts.
#[derive(BorshSerialize, BorshDeserialize)]
struct AppSchema {
    prices: [StoredPrice; 5],
    average_price: f32,
}

/// Element of prices array. If "is_some" == false then price is None
/// and price is not used in calculating the average price
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Copy, Clone)]
pub struct StoredPrice {
    pub price: f32,
    pub is_some: bool,
}

/// Parses and returns the Solana yaml config on the system.
pub fn get_config() -> Result<yaml_rust::Yaml> {
    let path = match home::home_dir() {
        Some(mut path) => {
            path.push(".config/solana/cli/config.yml");
            path
        }
        None => {
            return Err(Error::ConfigReadError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "failed to locate homedir and thus can not locate solana config",
            )));
        }
    };
    let config = std::fs::read_to_string(path).map_err(|e| Error::ConfigReadError(e))?;
    let mut config = YamlLoader::load_from_str(&config)?;
    match config.len() {
        1 => Ok(config.remove(0)),
        l => Err(Error::InvalidConfig(format!(
            "expected one yaml document got ({})",
            l
        ))),
    }
}

/// Gets the RPC url for the cluster that this machine is configured
/// to communicate with.
pub fn get_rpc_url() -> Result<String> {
    let config = get_config()?;
    match config["json_rpc_url"].as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(Error::InvalidConfig(
            "missing `json_rpc_url` field".to_string(),
        )),
    }
}

/// Gets the "player" or local solana wallet that has been configured
/// on the machine.
pub fn get_player() -> Result<Keypair> {
    let config = get_config()?;
    let path = match config["keypair_path"].as_str() {
        Some(s) => s,
        None => {
            return Err(Error::InvalidConfig(
                "missing `keypair_path` field".to_string(),
            ))
        }
    };
    read_keypair_file(path).map_err(|e| {
        Error::InvalidConfig(format!("failed to read keypair file ({}): ({})", path, e))
    })
}

/// Gets the seed used to generate price_sender accounts. If you'd like to
/// force this program to generate a new price_sender account and thus
/// restart the counter you can change this value.
pub fn get_index_seed() -> &'static str {
    "index"
}

/// Derives and returns the price sender account public key for a given
/// PLAYER, PROGRAM combination.
pub fn get_public_key(player: &Pubkey, program: &Pubkey) -> Result<Pubkey> {
    Ok(Pubkey::create_with_seed(
        player,
        get_index_seed(),
        program,
    )?)
}

/// Determines and reports the size of index schema data.
pub fn get_data_size() -> Result<usize> {
    let stored_prices = [StoredPrice::default(),
        StoredPrice::default(),
        StoredPrice::default(),
        StoredPrice::default(),
        StoredPrice::default()];
    let encoded = AppSchema { prices: stored_prices, average_price: f32::default() }
        .try_to_vec()
        .map_err(|e| Error::SerializationError(e))?;
    Ok(encoded.len())
}

/// Deserializes a index account and get average price
pub fn get_average_price(data: &[u8]) -> Result<f32> {
    let decoded: AppSchema = AppSchema::try_from_slice(data).map_err(|e| Error::SerializationError(e))?;
    Ok(decoded.average_price)
}
