//! Queries for the [Distribution module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/distribution/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`DistributionQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::cosmos_modules::distribution;

use super::QueryClient;

/// The distribution module's query client proto definition
pub type DistributionQueryClient = distribution::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for DistributionQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
