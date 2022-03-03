use crate::{
    chain_registry::get_chain,
    config::ChainClientConfig,
    error::{ChainClientError, RpcError},
};
use futures::executor;
use tendermint_rpc;

pub mod query;

type RpcClient = tendermint_rpc::HttpClient;

pub struct ChainClient {
    pub config: ChainClientConfig,
    // keybase: ?
    // KeyringOptions: Vec<?>
    pub rpc_client: RpcClient,
    // light_provider: ?
    // input:
    // output:
    // codec: ? // needed?
    // logger needed? i think rust does logging differently
}

impl ChainClient {
    pub fn new(chain_name: &str) -> Result<Self, ChainClientError> {
        let chain = executor::block_on(async { get_chain(chain_name).await })?;
        let config = chain.get_chain_config()?;
        let rpc_client = new_rpc_client(config.rpc_address.as_str())?;

        Ok(ChainClient { config, rpc_client })
    }
}

pub fn new_rpc_client(address: &str) -> Result<RpcClient, RpcError> {
    tendermint_rpc::HttpClient::new(address).map_err(|e| e.into())
}
