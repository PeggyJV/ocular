//! Queries for the [Slashing module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/slashing/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`SlashingQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmos_modules::slashing;

use super::GrpcClient;

/// The slashing module's query client proto definition
pub type SlashingQueryClient = slashing::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for SlashingQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        SlashingQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}
