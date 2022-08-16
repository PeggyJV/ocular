use serde::{Deserialize, Serialize};

use crate::tx::Coin;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChainClientConfig {
    pub chain_id: String,
    pub chain_name: String,
    #[serde(rename = "rpc-addr")]
    pub rpc_address: String,
    /// If you do not wish to manually specify a grpc address, simply provide an empty string: ""
    #[serde(rename = "grpc-addr")]
    pub grpc_address: String,
    pub account_prefix: String,
    pub gas_adjustment: f64,
    pub default_fee: Coin,
}
