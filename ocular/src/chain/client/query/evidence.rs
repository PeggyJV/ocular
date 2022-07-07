//! Queries for the [Evidence module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/evidence/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`EvidenceQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::cosmos_modules::evidence;

use super::QueryClient;

pub type EvidenceQueryClient = evidence::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for EvidenceQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}
