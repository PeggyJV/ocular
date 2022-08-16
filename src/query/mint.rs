//! Queries for the [Mint module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/mint/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`MintQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmos_modules::mint;

use super::GrpcClient;

/// The mint module's query client proto definition
pub type MintQueryClient = mint::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for MintQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        MintQueryClient::connect(endpoint).await
            .wrap_err("Failed to make gRPC connection")
    }
}
