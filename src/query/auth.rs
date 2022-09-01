//! Query methods for the [Auth module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/auth/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Report, Result};
use prost::Message;
use tonic::transport::channel::Channel;

use crate::cosmrs::{
    crypto::PublicKey,
    proto::cosmos::auth::v1beta1::{self as auth, QueryAccountsResponse},
};

use super::{GrpcClient, PageRequest, PageResponse, QueryClient};

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
    ) -> Result<AccountsResponse> {
        let query_client = self.get_grpc_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountsRequest { pagination };
        query_client
            .accounts(request)
            .await?
            .into_inner()
            .try_into()
    }

    /// Gets the auth module's params
    pub async fn auth_params(&mut self) -> Result<auth::QueryParamsResponse> {
        let query_client = self.get_grpc_query_client::<AuthQueryClient>().await?;
        let request = auth::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }
}

/// Convenience type for `all_accounts()` responses
#[derive(Clone, Debug)]
pub struct AccountsResponse {
    pub accounts: Vec<BaseAccount>,
    pub pagination: Option<PageResponse>,
}

impl TryFrom<QueryAccountsResponse> for AccountsResponse {
    type Error = Report;

    fn try_from(response: QueryAccountsResponse) -> Result<Self> {
        let base_accounts = response
            .accounts
            .iter()
            .map(|any| auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
            .collect::<Vec<auth::BaseAccount>>();
        let mut accounts = Vec::<BaseAccount>::new();
        for ba in base_accounts {
            accounts.push(ba.try_into()?)
        }

        Ok(Self {
            accounts,
            pagination: response.pagination,
        })
    }
}

/// Used for converting the BaseAccount type in cosmos_sdk_proto to something with concrete field types
#[derive(Clone, Debug)]
pub struct BaseAccount {
    pub address: String,
    // public key may not be present on chain
    pub pub_key: Option<PublicKey>,
    pub account_number: u64,
    pub sequence: u64,
}

// TO-DO: Handle public keys with type URL /cosmos.crypto.multisig.LegacyAminoPubKey
impl TryFrom<cosmrs::proto::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = Report;

    fn try_from(account: cosmrs::proto::cosmos::auth::v1beta1::BaseAccount) -> Result<BaseAccount> {
        let pub_key = match account.pub_key {
            Some(k) => Some(PublicKey::try_from(k)?),
            None => None,
        };

        Ok(BaseAccount {
            address: account.address,
            pub_key,
            account_number: account.account_number,
            sequence: account.sequence,
        })
    }
}
