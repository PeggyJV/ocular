use thiserror::Error;

// Higher level errors: ChainClientError, ChainInfoError, ChainRegistryError
#[derive(Debug, Error)]
pub enum ChainClientError {
    #[error("{0}")]
    ChainInfo(#[from] ChainInfoError),
    #[error("{0}")]
    ChainRegistry(#[from] ChainRegistryError),
    #[error("{0}")]
    Keyring(#[from] KeyStoreError),
    #[error("{0}")]
    Grpc(#[from] GrpcError),
    #[error("{0}")]
    ModuleQuery(String),
    #[error("error during RPC call: {0}")]
    Rpc(#[from] RpcError),
    #[error("{0}")]
    TxError(#[from] TxError),
    #[error("{0}")]
    AutomatedTxHandlerError(#[from] AutomatedTxHandlerError),
}

#[derive(Debug, Error)]
pub enum ChainInfoError {
    #[error("error during chain registry interaction: {0}")]
    ChainRegistry(#[from] ChainRegistryError),
    #[error("invalid gRPC endpoint(s): {0}")]
    GrpcEndpoint(#[from] GrpcError),
    #[error("invalid RPC endpoint(s): {0}")]
    RpcEndpoint(#[from] RpcError),
}

#[derive(Debug, Error)]
pub enum ChainRegistryError {
    #[error("error parsing chain info: {0}")]
    InvalidChainInfo(#[from] serde_json::error::Error),
    #[error("error during chain registry request: {0}")]
    Request(#[from] octocrab::Error),
    #[error("{0}")]
    UnsupportedChain(String),
}

// Lower level errors; should be used by higher level errors
#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("{0}")]
    Connection(#[from] tonic::transport::Error),
    #[error("{0}")]
    MissingEndpoint(String),
    #[error("{0}")]
    Request(#[from] tonic::Status),
    #[error("unhealthy gRPC endpoint: {0}")]
    UnhealthyEndpoint(String),
}

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("{0}")]
    MissingEndpoint(String),
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

    #[error("error reading file: {0}")]
    FileIO(String),
}

#[derive(Debug, Error)]
pub enum TxError {
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("error converting types: {0}")]
    TypeConversionError(String),
    #[error("error signing message: {0}")]
    SigningError(String),
    #[error("error broadcasting message: {0}")]
    BroadcastError(String),
    #[error("error logging response: {0}")]
    Logging(String),
}

#[derive(Debug, Error)]
pub enum AutomatedTxHandlerError {
    #[error("error reading file: {0}")]
    FileIO(String),
    #[error("error reading toml: {0}")]
    Toml(String),
    #[error("error handling key: {0}")]
    KeyHandling(String),
    #[error("error sending tx: {0}")]
    TxBroadcast(String),
    #[error("keystore error: {0}")]
    KeyStore(String),
    #[error("chain client error: {0}")]
    ChainClient(String),
    #[error("no valid unexpired authorization grants found for msg type: {0}")]
    Authorization(String),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("error processing file: {0}")]
    FileIO(String),
    #[error("error intializing cache: {0}")]
    Initialization(String),
}
