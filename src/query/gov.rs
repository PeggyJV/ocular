//! Queries for the [Gov module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/gov/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`GovQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::gov::v1beta1 as gov;

use super::GrpcClient;

/// The gov module's query client proto definition
pub type GovQueryClient = gov::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for GovQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        GovQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}
