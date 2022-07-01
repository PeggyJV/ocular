//! Queries for the [Gov module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/gov/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`GovQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::{cosmos_modules::gov};

use super::QueryClient;

pub type GovQueryClient = gov::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for GovQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
