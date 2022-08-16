//! Query methods for the [Auth module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/auth/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use prost::Message;
use tonic::transport::channel::Channel;

use crate::{account::BaseAccount, cosmos_modules::auth};

use super::{GrpcClient, PageRequest, QueryClient};

/// The auth module's query client proto definition
pub type AuthQueryClient = auth::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for AuthQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        AuthQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Gets the account on chain with the specified address
    pub async fn account(&mut self, address: &str) -> Result<BaseAccount> {
        let query_client = self.get_grpc_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountRequest {
            address: address.to_string(),
        };
        let response = query_client.account(request).await?.into_inner();
        let any = response.account.unwrap();

        auth::BaseAccount::decode(&any.value as &[u8])
            .unwrap()
            .try_into()
    }

    /// Gets all accounts
    pub async fn all_accounts(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<Vec<BaseAccount>> {
        let query_client = self.get_grpc_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountsRequest { pagination };
        let base_accounts = query_client
            .accounts(request)
            .await?
            .into_inner()
            .accounts
            .iter()
            .map(|any| auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
            .collect::<Vec<auth::BaseAccount>>();
        let mut accounts = Vec::<BaseAccount>::new();

        for ba in base_accounts {
            accounts.push(ba.try_into()?)
        }

        Ok(accounts)
    }
}
