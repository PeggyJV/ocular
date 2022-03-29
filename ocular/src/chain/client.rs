#![warn(unused_qualifications)]

use crate::{
    chain::{config::ChainClientConfig, registry::get_chain},
    error::{ChainClientError, RpcError},
    keyring::Keyring,
};
use futures::executor;
use tendermint_rpc;

use super::ChainName;

pub mod authz;
pub mod query;
pub mod tx;

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
    pub fn new(chain_name: ChainName) -> Result<Self, ChainClientError> {
        get_client(chain_name.as_str())
    }
}

fn get_client(chain_name: &str) -> Result<ChainClient, ChainClientError> {
    let chain = executor::block_on(async { get_chain(chain_name).await })?;
    let config = chain.get_chain_config()?;
    let keyring = Keyring::new_file_store(None)?;
    let rpc_client = new_rpc_client(config.rpc_address.as_str())?;

    Ok(ChainClient {
        config,
        keyring,
        rpc_client,
    })
}

pub fn new_rpc_client(address: &str) -> Result<RpcClient, RpcError> {
    tendermint_rpc::HttpClient::new(address).map_err(|e| e.into())
}
