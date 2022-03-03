use crate::error::{ChainClientError, RpcError};
use tendermint_rpc::Client;

use super::ChainClient;

impl ChainClient {
    pub async fn query_latest_height(&self) -> Result<u64, ChainClientError> {
        let status = self
            .rpc_client
            .status()
            .await
            .map_err(RpcError::TendermintStatus)?;
        Ok(status.sync_info.latest_block_height.value())
    }
}
