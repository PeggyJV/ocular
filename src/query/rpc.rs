use eyre::Result;
use tendermint_rpc::Client as TendermintClient;

use super::QueryClient;

impl QueryClient {
    /// RPC query for latest block height
    pub async fn latest_height(&self) -> Result<u64> {
        let status = self
            .rpc_client
            .status()
            .await?;
        Ok(status.sync_info.latest_block_height.value())
    }
}
