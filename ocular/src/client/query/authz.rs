//! Query methods for the [Authz module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/authz/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthzQueryClient`] directly.
use crate::{
    cosmos_modules::authz::{self, *},
    error::{ChainClientError, GrpcError}, client::ChainClient,
};
use tonic::transport::Channel;

use super::GrpcClient;

/// The authz module's query client proto definition
pub type AuthzQueryClient = authz::query_client::QueryClient<Channel>;
pub type Grant = authz::Grant;

impl GrpcClient for AuthzQueryClient {}

impl ChainClient {
    /// Gets all grants between `granter` and `grantee` for the given msg type
    pub async fn query_authz_grant(
        &mut self,
        granter: &str,
        grantee: &str,
        msg_type_url: &str,
    ) -> Result<Vec<Grant>, ChainClientError> {
        let query_client = self.get_grpc_query_client::<AuthzQueryClient>().await?;
        let request = QueryGrantsRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: msg_type_url.to_string(),
            // TODO: Support pagination if use case arises
            pagination: None,
        };

        Ok(query_client
            .grants(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner()
            .grants)
    }
}
