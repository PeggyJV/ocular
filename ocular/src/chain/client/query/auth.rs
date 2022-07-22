//! Query methods for the [Auth module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/auth/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthQueryClient`] directly.
use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use prost::Message;
use tonic::transport::Channel;

use crate::{
    account::BaseAccount,
    cosmos_modules::auth,
    error::{ChainClientError, GrpcError},
};

use super::{ChainClient, QueryClient};

/// The auth module's query client proto definition
pub type AuthQueryClient = auth::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for AuthQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}

impl ChainClient {
    /// Gets the account on chain with the specified address
    pub async fn query_account(&mut self, address: &str) -> Result<BaseAccount, ChainClientError> {
        let mut query_client = self.get_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountRequest {
            address: address.to_string(),
        };
        let response = query_client
            .account(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        let any = response.account.unwrap();

        Ok(auth::BaseAccount::decode(&any.value as &[u8])
            .unwrap()
            .try_into()?)
    }

    /// Gets all accounts
    pub async fn query_accounts(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<Vec<BaseAccount>, ChainClientError> {
        let mut query_client = self.get_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountsRequest { pagination };
        let base_accounts = query_client
            .accounts(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner()
            .accounts
            .iter()
            .map(|any| {
                auth::BaseAccount::decode(&any.value as &[u8])
                    .unwrap()
            })
            .collect::<Vec<auth::BaseAccount>>();
        let mut accounts = Vec::<BaseAccount>::new();

        for ba in base_accounts {
            accounts.push(ba.try_into()?)
        }

        Ok(accounts)
    }
}
