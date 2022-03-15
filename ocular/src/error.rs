use thiserror::Error;

// Higher level errors: ChainClientError, ChainInfoError, ChainRegistryError
#[derive(Debug, Error)]
pub enum ChainClientError {
    #[error("{0}")]
    ChainInfo(#[from] ChainInfoError),
    #[error("{0}")]
    ChainRegistry(#[from] ChainRegistryError),
    #[error("error during RPC call: {0}")]
    TendermintRpc(#[from] RpcError),
}

#[derive(Debug, Error)]
pub enum ChainInfoError {
    #[error("error during chain registry interaction: {0}")]
    ChainRegistry(#[from] ChainRegistryError),
    #[error("invalid RPC endpoint(s): {0}")]
    RpcEndpoint(#[from] RpcError),
}

#[derive(Debug, Error)]
pub enum ChainRegistryError {
    #[error("error parsing chain info: {0}")]
    InvalidChainInfo(#[from] serde_json::error::Error),
    #[error("error during chain registry request: {0}")]
    Request(#[from] octocrab::Error),
}

// Lower level errors; should be used by higher level errors
#[derive(Debug, Error)]
pub enum RpcError {
    #[error("tendermint status error: {0}")]
    TendermintStatus(#[from] tendermint_rpc::Error),
    #[error("unhealthy RPC endpoint: {0}")]
    UnhealthyEndpoint(String),
}

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("error creating or opening keystore: {0}")]
    CouldNotOpenOrCreateKeyStore(String),

    #[error("key name '{0}' already exists.")]
    Exists(String),

    #[error("key name '{0}' does not exist.")]
    DoesNotExist(String),

    #[error("key store has not been initialized.")]
    NotInitialized,

    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("unable to store key: {0}")]
    UnableToStoreKey(String),

    #[error("unable to delete key: {0}")]
    UnableToDeleteKey(String),

    #[error("unable to retrieve key: {0}")]
    UnableToRetrieveKey(String),
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("error converting types: {0}")]
    TypeConversionError(String),

    #[error("error signing message: {0}")]
    SigningError(String),
}
