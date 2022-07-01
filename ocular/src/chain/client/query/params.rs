//! Queries for the [Params module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/params/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`ParamsQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::{cosmos_modules::params};

use super::QueryClient;

pub type ParamsQueryClient = params::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for ParamsQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
