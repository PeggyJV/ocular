use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChainClientConfig {
    pub chain_id: String,
    #[serde(rename = "rpc-addr")]
    pub rpc_address: String,
    #[serde(rename = "grpc-addr")]
    pub grpc_address: String,
    pub account_prefix: String,
    pub gas_adjustment: f64,
    pub gas_prices: String,
}

/*
    ChainID        string                  `json:"chain-id" yaml:"chain-id"`
    RPCAddr        string                  `json:"rpc-addr" yaml:"rpc-addr"`
    GRPCAddr       string                  `json:"grpc-addr" yaml:"grpc-addr"`
    AccountPrefix  string                  `json:"account-prefix" yaml:"account-prefix"`
    GasAdjustment  float64                 `json:"gas-adjustment" yaml:"gas-adjustment"`
    GasPrices      string                  `json:"gas-prices" yaml:"gas-prices"`
    TrustingPeriod string                  `json:"omitempty" yaml:"omitempty"`
*/
