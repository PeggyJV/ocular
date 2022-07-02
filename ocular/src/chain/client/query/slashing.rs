//! Queries for the [Slashing module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/slashing/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`SlashingQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::cosmos_modules::slashing;

use super::QueryClient;

pub type SlashingQueryClient = slashing::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for SlashingQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
