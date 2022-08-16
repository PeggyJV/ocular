//! Queries for the [Staking module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/staking/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`StakingQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmos_modules::staking;

use super::GrpcClient;

/// The staking module's query client proto definition
pub type StakingQueryClient = staking::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for StakingQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        StakingQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}
