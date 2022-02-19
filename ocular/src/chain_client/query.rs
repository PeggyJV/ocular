use tendermint_rpc::Client;

use crate::error::{ChainClientError, RpcError};

use super::ChainClient;

impl ChainClient {
    pub async fn query_latest_height(&self) -> Result<u64, ChainClientError> {
        let status = self
            .rpc_client
            .status()
            .await
            .map_err(|e| RpcError::TendermintStatus(e))?;
        Ok(status.sync_info.latest_block_height.value())
    }
}
