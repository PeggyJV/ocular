//! Query methods for the [Auth module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/auth/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthQueryClient`] directly.
use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use prost::Message;
use tonic::transport::Channel;

use crate::{
    cosmos_modules::auth,
    error::{ChainClientError, GrpcError},
};

use super::{ChainClient, QueryClient};

pub type AuthQueryClient = auth::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for AuthQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}

impl ChainClient {
    pub async fn query_account(
        &mut self,
        address: &str,
    ) -> Result<auth::BaseAccount, ChainClientError> {
        let mut query_client = self.get_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountRequest { address: address.to_string() };
        let response = query_client
            .account(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        let any = response.account.unwrap();

        Ok(auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
    }

    /// Gets all accounts
    pub async fn query_accounts(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<Vec<auth::BaseAccount>, ChainClientError> {
        let mut query_client = self.get_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountsRequest { pagination };

        Ok(query_client
            .accounts(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner()
            .accounts
            .iter()
            .map(|any| auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
            .collect())
    }
}
