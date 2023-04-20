//! Queries and messages for the [Params module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/params/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`ParamsQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::params::v1beta1 as params;

use super::{ConstructClient, GrpcClient};

/// The params module's query client proto definition
pub type ParamsQueryClient = params::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<ParamsQueryClient> for ParamsQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        ParamsQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets the chain's params
    pub async fn query_params(
        &mut self,
        subspace: &str,
        key: &str,
    ) -> Result<params::QueryParamsResponse> {
        let query_client = self.get_client::<ParamsQueryClient>().await?;
        let request = params::QueryParamsRequest {
            subspace: subspace.to_string(),
            key: key.to_string(),
        };

        Ok(query_client.params(request).await?.into_inner())
    }
}