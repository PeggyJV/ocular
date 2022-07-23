//! Queries for the [Staking module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/staking/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`StakingQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::cosmos_modules::staking;

use super::QueryClient;

/// The staking module's query client proto definition
pub type StakingQueryClient = staking::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for StakingQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
