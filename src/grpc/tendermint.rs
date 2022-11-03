//! Queries and messages for [Tendermint](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/base/tendermint/v1beta1/query.proto). Consider also using the [tendermint-rpc](https://crates.io/crates/tendermint-rpc) crate if these methods don't meet your requirements.
use async_trait::async_trait;
use cosmrs::proto::cosmos::base::tendermint::v1beta1::*;
use eyre::{Context, Result};
use tonic::transport::Channel;

use super::{ConstructClient, GrpcClient, PageRequest};

/// Tendermint's query client proto definition
pub type TendermintServiceClient =
    crate::cosmrs::proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient<Channel>;

#[async_trait]
impl ConstructClient<TendermintServiceClient> for TendermintServiceClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        TendermintServiceClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets the node info
    pub async fn query_node_info(&mut self) -> Result<GetNodeInfoResponse> {
        let query_client = self.get_client::<TendermintServiceClient>().await?;

        Ok(query_client
            .get_node_info(GetNodeInfoRequest {})
            .await?
            .into_inner())
    }

    /// Gets the latest block
    pub async fn query_latest_block(&mut self) -> Result<GetLatestBlockResponse> {
        let query_client = self.get_client::<TendermintServiceClient>().await?;

        Ok(query_client
            .get_latest_block(GetLatestBlockRequest {})
            .await?
            .into_inner())
    }

    /// Indicates whether the node is syncing
    pub async fn query_syncing(&mut self) -> Result<bool> {
        let query_client = self.get_client::<TendermintServiceClient>().await?;

        Ok(query_client
            .get_syncing(GetSyncingRequest {})
            .await?
            .into_inner()
            .syncing)
    }

    /// Gets the block at the specified height
    pub async fn query_block_by_height(&mut self, height: u64) -> Result<GetBlockByHeightResponse> {
        let query_client = self.get_client::<TendermintServiceClient>().await?;
        let request = GetBlockByHeightRequest {
            height: height as i64,
        };

        Ok(query_client
            .get_block_by_height(request)
            .await?
            .into_inner())
    }

    /// Gets the latest validator set
    pub async fn query_latest_validator_set(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<GetLatestValidatorSetResponse> {
        let query_client = self.get_client::<TendermintServiceClient>().await?;
        let request = GetLatestValidatorSetRequest { pagination };

        Ok(query_client
            .get_latest_validator_set(request)
            .await?
            .into_inner())
    }

    /// Gets the validator set at the specified height
    pub async fn query_validator_set_by_height(
        &mut self,
        height: u64,
        pagination: Option<PageRequest>,
    ) -> Result<GetValidatorSetByHeightResponse> {
        let query_client = self.get_client::<TendermintServiceClient>().await?;
        let request = GetValidatorSetByHeightRequest {
            height: height as i64,
            pagination,
        };

        Ok(query_client
            .get_validator_set_by_height(request)
            .await?
            .into_inner())
    }
}
