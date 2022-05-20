use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainClientConfig {
    pub chain_name: String,
    pub chain_id: String,
    #[serde(rename = "rpc-addr")]
    pub rpc_address: String,
    pub account_prefix: String,
    pub gas_adjustment: f64,
    pub gas_prices: String,
}
