use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChainClientConfig {
    pub key: String,
    pub chain_id: String,
    #[serde(rename = "rpc-addr")]
    pub rpc_address: String,
    #[serde(rename = "grpc-addr")]
    pub grpc_address: String,
    pub account_prefix: String,
    pub keyring_backend: String,
    pub gas_adjustment: f64,
    pub gas_prices: String,
    pub key_directory: String,
}

/*  	Key            string                  `json:"key" yaml:"key"`
    ChainID        string                  `json:"chain-id" yaml:"chain-id"`
    RPCAddr        string                  `json:"rpc-addr" yaml:"rpc-addr"`
    GRPCAddr       string                  `json:"grpc-addr" yaml:"grpc-addr"`
    AccountPrefix  string                  `json:"account-prefix" yaml:"account-prefix"`
    KeyringBackend string                  `json:"keyring-backend" yaml:"keyring-backend"`
    GasAdjustment  float64                 `json:"gas-adjustment" yaml:"gas-adjustment"`
    GasPrices      string                  `json:"gas-prices" yaml:"gas-prices"`
    TrustingPeriod string                  `json:"omitempty" yaml:"omitempty"`
    KeyDirectory   string                  `json:"key-directory" yaml:"key-directory"`
*/
