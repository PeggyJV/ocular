//! Queries for the [Evidence module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/evidence/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`EvidenceQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmos_modules::evidence;

use super::GrpcClient;

/// The evidence module's query client proto definition
pub type EvidenceQueryClient = evidence::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for EvidenceQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        EvidenceQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}
