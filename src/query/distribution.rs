//! Queries for the [Distribution module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/distribution/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`DistributionQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmos_modules::distribution;

use super::GrpcClient;

/// The distribution module's query client proto definition
pub type DistributionQueryClient = distribution::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for DistributionQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        DistributionQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}
