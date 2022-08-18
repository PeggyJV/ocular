//! Query methods for the [Authz module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/authz/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthzQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmos_modules::authz::{self, *};

use super::{GrpcClient, QueryClient, PageRequest};

/// The authz module's query client proto definition
pub type AuthzQueryClient = authz::query_client::QueryClient<Channel>;
pub type Grant = authz::Grant;

#[async_trait]
impl GrpcClient for AuthzQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        AuthzQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Gets all grants between `granter` and `grantee` for the given msg type
    pub async fn grants(
        &mut self,
        granter: &str,
        grantee: &str,
        msg_type_url: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryGrantsResponse> {
        let query_client = self.get_grpc_query_client::<AuthzQueryClient>().await?;
        let request = QueryGrantsRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: msg_type_url.to_string(),
            // TODO: Support pagination if use case arises
            pagination,
        };

        Ok(query_client.grants(request).await?.into_inner())
    }

    /// Gets all grant authorizations granted by the provided `granter`
    pub async fn granter_grants(
        &mut self,
        granter: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryGranterGrantsResponse> {
        let query_client = self.get_grpc_query_client::<AuthzQueryClient>().await?;
        let request = QueryGranterGrantsRequest {
            granter: granter.to_string(),
            pagination,
        };

        Ok(query_client.granter_grants(request).await?.into_inner())
    }

    /// Gets all grant authorizations granted to the provided `grantee`
    pub async fn grantee_grants(
        &mut self,
        grantee: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryGranteeGrantsResponse> {
        let query_client = self.get_grpc_query_client::<AuthzQueryClient>().await?;
        let request = QueryGranteeGrantsRequest {
            grantee: grantee.to_string(),
            pagination,
        };

        Ok(query_client.grantee_grants(request).await?.into_inner())
    }
}
