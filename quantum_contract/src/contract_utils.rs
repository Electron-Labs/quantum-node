use anyhow::{anyhow, Result as AnyhowResult};
use quantum_utils::keccak::decode_keccak_hex;
use tracing::{info, error};

pub fn get_f64_from_json_value_object(json_value: serde_json::Value) -> Option<f64> {
    Some(json_value.as_number()?.as_f64()?)
}

// gas cost in gwei
pub async fn get_gas_cost() -> AnyhowResult<f64> {
    let gas_cost_rpc = &std::env::var("GAS_COST_RPC")?;
    let gas_cost_api_key = &std::env::var("GAS_COST_API_KEY")?;

    info!("{}", format!("Fetching Gas Cost: {gas_cost_rpc}"));
    let client = reqwest::Client::new();
    let res = client.get(gas_cost_rpc).header("Authorization", gas_cost_api_key).send().await?;

    let json: serde_json::Value = res.json().await?;
    let block_prices = match json["blockPrices"].as_array() {
        Some(t) => Ok(t),
        None => {
            error!("not able to find block prices in fetching gas cost response");
            Err(anyhow!("not able to find block prices in fetching gas cost response"))
        },
    }?;
    let base_fees = get_f64_from_json_value_object(block_prices[0]["baseFeePerGas"].clone());
    let base_fees = match base_fees {
        Some(fee) => Ok(fee),
        None => {
            error!("not able to get the base fee from get gas api");
            Err(anyhow!("not able to get the base fee from get gas api"))
        },
    }?;
    Ok(base_fees)
}

// eth_price in USD
pub async fn get_eth_price() -> AnyhowResult<f64> {
    let eth_price_rpc = &std::env::var("ETH_PRICE_RPC")?;
    info!("{}", format!("Fetching Ethereum Price: {eth_price_rpc}"));

    let res = reqwest::get(eth_price_rpc).await?;
    let json: serde_json::Value = res.json().await?;
    info!("json: {:?}", json);

    let usd_price = get_f64_from_json_value_object(json["USD"].clone());
    let usd_price = match usd_price {
        Some(fee) => Ok(fee),
        None => {
            error!("not able to get the base fee from get gas api");
            Err(anyhow!("not able to get the base fee from get gas api"))
        },
    }?;
    Ok(usd_price)
}

pub fn get_bytes_from_hex_string(value: &str) ->AnyhowResult<[u8; 32]> {
    Ok(decode_keccak_hex(value)?)
}