//! Queries for the [Params module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/params/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`ParamsQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::params::v1beta1 as params;

use super::{GrpcClient, QueryClient};

/// The params module's query client proto definition
pub type ParamsQueryClient = params::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for ParamsQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        ParamsQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    pub async fn params(
        &mut self,
        subspace: &str,
        key: &str,
    ) -> Result<params::QueryParamsResponse> {
        let query_client = self.get_grpc_query_client::<ParamsQueryClient>().await?;
        let request = params::QueryParamsRequest {
            subspace: subspace.to_string(),
            key: key.to_string(),
        };

        Ok(query_client.params(request).await?.into_inner())
    }
}
