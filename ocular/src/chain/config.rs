use serde::{Deserialize, Serialize};
use crate::chain::ChainName;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainClientConfig {
    pub chain_name: ChainName,
    pub chain_id: String,
    #[serde(rename = "rpc-addr")]
    pub rpc_address: String,
    pub account_prefix: String,
    pub gas_adjustment: f64,
    pub gas_prices: String,
}
