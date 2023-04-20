//! Queries and messages for [Tx](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/tx/v1beta1/tx.proto). Consider also using the [tendermint-rpc](https://crates.io/crates/tendermint-rpc) crate if these methods don't meet your requirements.
use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::*;
use eyre::{Context, Result};

use crate::tx::*;

use super::{ConstructClient, GrpcClient, TxClient};

#[async_trait]
impl ConstructClient<TxClient> for TxClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        TxClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets a tx by its hash. Will error if the hash is not found.
    pub async fn query_tx_by_hash(&mut self, hash: &str) -> Result<GetTxResponse> {
        let client = self.get_client::<TxClient>().await?;
        let request = GetTxRequest {
            hash: hash.to_string(),
        };

        Ok(client.get_tx(request).await?.into_inner())
    }

    /// Broadcasts transaction using the /broadcast_async Tendermint endpoint. Returns right away without waiting on CheckTx.
    pub async fn broadcast_async(&mut self, tx: SignedTx) -> Result<BroadcastTxResponse> {
        let request = BroadcastTxRequest {
            tx_bytes: tx.to_bytes()?,
            mode: BroadcastMode::Async.into(),
        };

        Ok(self.tx_client.broadcast_tx(request).await?.into_inner())
    }

    /// Broadcasts transaction using the /broadcast_commit Tendermint endpoint, waiting for CheckTx and DeliverTx to complete
    /// before returning. Note that the server may time out the connection while waiting for the tx to be included in a block.
    /// This can result in an error being returned by this method even if the tx is ultimately successful.
    pub async fn broadcast_commit(&mut self, tx: SignedTx) -> Result<BroadcastTxResponse> {
        let request = BroadcastTxRequest {
            tx_bytes: tx.to_bytes()?,
            mode: BroadcastMode::Block.into(),
        };

        Ok(self.tx_client.broadcast_tx(request).await?.into_inner())
    }

    /// Broadcasts transaction using the /broadcast_sync Tendermint endpoint. Waits for CheckTx but not DeliverTx.
    pub async fn broadcast_sync(&mut self, tx: SignedTx) -> Result<BroadcastTxResponse> {
        let request = BroadcastTxRequest {
            tx_bytes: tx.to_bytes()?,
            mode: BroadcastMode::Sync.into(),
        };

        Ok(self.tx_client.broadcast_tx(request).await?.into_inner())
    }

    /// Simulates the execution of a transaction, providing an estimate of gas usage info.
    pub async fn simulate(&mut self, tx: SignedTx) -> Result<SimulateResponse> {
        let request = SimulateRequest {
            tx_bytes: tx.to_bytes()?,
            ..Default::default()
        };

        Ok(self.tx_client.simulate(request).await?.into_inner())
    }
}
