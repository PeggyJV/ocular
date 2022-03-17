#![warn(unused_qualifications)]

use crate::{
    chain::{config::ChainClientConfig, registry::get_chain},
    error::{ChainClientError, RpcError},
    keyring::Keyring,
};
use futures::executor;
use tendermint_rpc;

pub mod query;
pub mod staking;
pub mod transactions;

type RpcClient = tendermint_rpc::HttpClient;

pub struct ChainClient {
    pub config: ChainClientConfig,
    pub keyring: Keyring,
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
        let keyring = Keyring::new_file_store(None).expect("Could not create keyring.");

        Ok(ChainClient {
            config,
            keyring,
            rpc_client,
        })
    }
}

pub fn new_rpc_client(address: &str) -> Result<RpcClient, RpcError> {
    tendermint_rpc::HttpClient::new(address).map_err(|e| e.into())
}
