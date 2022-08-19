//! Queries for the [Evidence module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/evidence/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`EvidenceQueryClient`] directly.
use async_trait::async_trait;
use cosmrs::proto::cosmos::evidence::v1beta1::QueryAllEvidenceResponse;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::{cosmrs::proto::cosmos::evidence::v1beta1 as evidence};

use super::{GrpcClient, QueryClient, PageRequest};

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

impl QueryClient {
    /// Gets evidence with the specified hash. Hash must be a valid hex string.
    pub async fn evidence(&mut self, evidence_hash: String) -> Result<evidence::QueryEvidenceResponse> {
        let query_client = self.get_grpc_query_client::<EvidenceQueryClient>().await?;
        let request = evidence::QueryEvidenceRequest {
            evidence_hash: hex::decode(evidence_hash)?,
        };

        Ok(query_client
            .evidence(request)
            .await?
            .into_inner())
    }

    /// Gets all evidence with optional pagination
    pub async fn all_evidence(&mut self, pagination: Option<PageRequest>) -> Result<QueryAllEvidenceResponse> {
        let query_client = self.get_grpc_query_client::<EvidenceQueryClient>().await?;
        let request = evidence::QueryAllEvidenceRequest {
            pagination,
        };

        Ok(query_client
            .all_evidence(request)
            .await?
            .into_inner())
    }
}
