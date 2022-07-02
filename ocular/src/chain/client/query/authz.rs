//! Query methods for the [Authz module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/authz/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthzQueryClient`] directly.
use crate::{
    cosmos_modules::authz::{self, *},
    error::{ChainClientError, GrpcError},
};
use async_trait::async_trait;
use tonic::transport::Channel;

use super::{ChainClient, QueryClient};

pub type AuthzQueryClient = authz::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for AuthzQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}

impl ChainClient {
    // Query for a specific msg grant
    pub async fn query_authz_grant(
        &mut self,
        granter: &str,
        grantee: &str,
        msg_type_url: &str,
    ) -> Result<QueryGrantsResponse, ChainClientError> {
        let mut query_client = self.get_query_client::<AuthzQueryClient>().await?;

        let request = QueryGrantsRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: msg_type_url.to_string(),
            // TODO: Support pagination if use case arises
            pagination: None,
        };

        let response = query_client
            .grants(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response)
    }
}