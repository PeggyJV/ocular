use crate::{
    cosmos_modules::*,
    error::{ChainClientError, GrpcError},
};

use tonic::transport::Channel;

use super::ChainClient;

pub type TxClient = tx::service_client::ServiceClient<Channel>;

impl ChainClient {
    // TODO: remove this method copy once https://github.com/PeggyJV/ocular/pull/22 merged
    fn check_for_grpc_address(&self) -> Result<(), GrpcError> {
        if self.config.grpc_address.is_empty() {
            return Err(GrpcError::MissingEndpoint(format!(
                "no grpc address available for chain {}",
                self.config.chain_id
            )));
        }

        Ok(())
    }

    /// Get Transaction Service Client
    pub async fn get_tx_client(&self) -> Result<TxClient, GrpcError> {
        self.check_for_grpc_address()?;

        TxClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(GrpcError::Connection)
    }

    /// Takes in a message represented by a byte array and a broadcast mode (https://docs.cosmos.network/v0.42/core/proto-docs.html#cosmos.tx.v1beta1.BroadcastMode)
    pub async fn broadcast_message(
        &self,
        tx_bytes: Vec<u8>,
        broadcast_mode: tx::BroadcastMode,
    ) -> Result<tx::BroadcastTxResponse, ChainClientError> {
        let mut tx_client = self.get_tx_client().await?;

        let broadcast_request = tx::BroadcastTxRequest {
            tx_bytes,
            mode: broadcast_mode.into(),
        };

        Ok(tx_client
            .broadcast_tx(broadcast_request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner())
    }
}

// ---------------------------------- Tests ----------------------------------
// Super basic unit tests since these operations inherently require chains
#[cfg(test)]
mod tests {
    use crate::chain::client::ChainClient;
    use assay::assay;

    #[assay]
    async fn tx_client_initialization() {
        let client = ChainClient::new("cosmoshub").unwrap();

        client
            .get_tx_client()
            .await
            .expect("failed to get transaction client");
    }
}
