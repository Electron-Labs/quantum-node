use anyhow::{anyhow, Result as AnyhowResult};
use quantum_utils::{error_line, keccak::decode_keccak_hex};
use tracing::info;

pub fn get_f64_from_json_value_object(json_value: serde_json::Value) -> Option<f64> {
    Some(json_value.as_number()?.as_f64()?)
}

// gas cost in gwei
pub async fn get_gas_cost() -> AnyhowResult<f64> {
    let gas_cost_rpc = &std::env::var("GAS_COST_RPC")?;
    let gas_cost_api_key = &std::env::var("GAS_COST_API_KEY")?;

    info!("{}", format!("Fetching Gas Cost: {gas_cost_rpc}"));
    let client = reqwest::Client::new();

    let mut base_fees: AnyhowResult<f64> = Err(anyhow!(error_line!("missing base fees")));

    for _ in 0..5 {
        let res = client
            .get(gas_cost_rpc)
            .header("Authorization", gas_cost_api_key)
            .send()
            .await?;
        let json: serde_json::Value = res.json().await?;
        match json["blockPrices"].as_array() {
            Some(block_prices) => {
                let base_fees_option =
                    get_f64_from_json_value_object(block_prices[0]["baseFeePerGas"].clone());
                match base_fees_option {
                    Some(some_base_fees) => {
                        base_fees = Ok(some_base_fees);
                        break;
                    },
                    None => {
                        info!("not able to get the base fee from get gas api. Trying again...");
                        base_fees = Err(anyhow!(error_line!(
                            "not able to get the base fee from get gas api"
                        )));
                        continue;
                    }
                };
            }
            None => {
                info!(
                    "not able to find block prices in fetching gas cost response. Trying again..."
                );
                base_fees = Err(anyhow!(error_line!(
                    "not able to find block prices in fetching gas cost response"
                )));
                continue;
            }
        };
    }

    base_fees
}

// eth_price in USD
pub async fn get_eth_price() -> AnyhowResult<f64> {
    let eth_price_rpc = &std::env::var("ETH_PRICE_RPC")?;
    info!("{}", format!("Fetching Ethereum Price: {eth_price_rpc}"));

    let mut usd_price: AnyhowResult<f64> = Err(anyhow!(error_line!("missing usd_price")));

    for _ in 0..5 {
        let res = reqwest::get(eth_price_rpc).await?;
        let json: serde_json::Value = res.json().await?;
        info!("eth_price_rpc json: {:?}", json);

        let usd_price_option = get_f64_from_json_value_object(json["USD"].clone());
        match usd_price_option {
            Some(some_usd_price) => {
                usd_price = Ok(some_usd_price);
                break;
            },
            None => {
                info!("not able to get the USD fee from get gas api. Trying again...");
                usd_price = Err(anyhow!(error_line!(
                    "not able to get the USD fee from get gas api"
                )));
                continue;
            }
        };
    }

    usd_price
}

pub fn get_bytes_from_hex_string(value: &str) ->AnyhowResult<[u8; 32]> {
    Ok(decode_keccak_hex(value)?)
}