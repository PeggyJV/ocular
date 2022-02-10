use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainClientError {
    #[error("error during key management operation: {0}")]
    Keys(#[from] signatory::Error)
}

#[derive(Debug, Error)]
pub enum ChainRegistryError {
    #[error("error parsing chain info: {0}")]
    InvalidChainInfo(#[from] serde_json::error::Error),
    #[error("error during chain registry request: {0}")]
    Request(#[from] octocrab::Error),
}

#[derive(Debug, Error)]
pub enum ChainInfoError {
    #[error("error during chain registry interaction: {0}")]
    ChainRegistry(#[from] ChainRegistryError),
    #[error("invalid RPC endpoint(s): {0}")]
    RpcEndpoint(String),
}
