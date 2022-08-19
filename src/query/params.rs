//! Queries for the [Params module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/params/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`ParamsQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::params::v1beta1 as params;

use super::GrpcClient;

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
