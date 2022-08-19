//! Queries for the [FeeGrant module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/feegrant/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`FeeGrantQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::feegrant::v1beta1::{self as feegrant, Grant, QueryAllowancesResponse};

use super::{GrpcClient, QueryClient, PageRequest};

/// The gov module's query client proto definition
pub type FeeGrantQueryClient = feegrant::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for FeeGrantQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        FeeGrantQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Allowance returns fee granted to the grantee by the granter.
    pub async fn allowance(&mut self, granter: &str, grantee: &str) -> Result<Option<Grant>> {
        let query_client = self.get_grpc_query_client::<FeeGrantQueryClient>().await?;
        let request = feegrant::QueryAllowanceRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
        };

        Ok(query_client
            .allowance(request)
            .await?
            .into_inner()
            .allowance)
    }

    /// Allowances returns all the grants for address.
    pub async fn all_allowances(&mut self, grantee: &str, pagination: Option<PageRequest>) -> Result<QueryAllowancesResponse> {
        let query_client = self.get_grpc_query_client::<FeeGrantQueryClient>().await?;
        let request = feegrant::QueryAllowancesRequest {
            grantee: grantee.to_string(),
            pagination
        };

        Ok(query_client
            .allowances(request)
            .await?
            .into_inner())
    }
}
