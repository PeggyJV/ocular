//! Queries for the [Mint module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/mint/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`MintQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::{cosmos_modules::mint};

use super::QueryClient;

pub type MintQueryClient = mint::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for MintQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
