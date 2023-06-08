//! Query methods for the [Auth module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/auth/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthQueryClient`] directly.
use async_trait::async_trait;
use cosmrs::Any;
use eyre::{Context, Report, Result};
use prost::Message;
use tonic::transport::channel::Channel;

use crate::cosmrs::{
    crypto::PublicKey,
    proto::cosmos::auth::v1beta1::{self as auth},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The auth module's query client proto definition
pub type AuthQueryClient = auth::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<AuthQueryClient> for AuthQueryClient {
    async fn new_client(endpoint: String) -> Result<AuthQueryClient> {
        AuthQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets the account on chain with the specified address
    pub async fn query_account(&mut self, address: &str) -> Result<BaseAccount> {
        let query_client = self.get_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountRequest {
            address: address.to_string(),
        };
        let response = query_client.account(request).await?.into_inner();
        let any = response.account.unwrap();

        auth::BaseAccount::decode(&any.value as &[u8])
            .unwrap()
            .try_into()
    }

    /// Gets the account on chain with the specified address as a raw [`cosmrs::Any`]
    pub async fn query_account_raw(&mut self, address: &str) -> Result<Any> {
        let query_client = self.get_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountRequest {
            address: address.to_string(),
        };
        let response = query_client.account(request).await?.into_inner();
        Ok(response.account.unwrap())
    }

    /// Gets all accounts
    pub async fn query_all_accounts(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<auth::QueryAccountsResponse> {
        let query_client = self.get_client::<AuthQueryClient>().await?;
        let request = auth::QueryAccountsRequest { pagination };
        Ok(query_client.accounts(request).await?.into_inner())
    }

    /// Gets the auth module's params
    pub async fn query_auth_params(&mut self) -> Result<auth::QueryParamsResponse> {
        let query_client = self.get_client::<AuthQueryClient>().await?;
        let request = auth::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }
}

/// Used for converting the BaseAccount type in cosmos_sdk_proto to something with concrete field types
#[derive(Clone, Debug)]
pub struct BaseAccount {
    /// Bech32 address of the account
    pub address: String,
    /// Public key of the account. May or may not be present on chain.
    pub pub_key: Option<PublicKey>,
    /// Account number
    pub account_number: u64,
    /// Sequence number representing how many previous transactions have been executed by the current
    /// account number used to prevent replay attacks.
    pub sequence: u64,
}

// TO-DO: Handle public keys with type URL /cosmos.crypto.multisig.LegacyAminoPubKey?
impl TryFrom<cosmrs::proto::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = Report;

    /// This will ignore public keys not of type `/cosmos.crypto.ed25519.PubKey` or `/cosmos.crypto.secp256k1.PubKey`
    fn try_from(account: cosmrs::proto::cosmos::auth::v1beta1::BaseAccount) -> Result<BaseAccount> {
        let pub_key = match account.pub_key {
            // We don't currently support LegacyAminoPubKey so we simply return None if decoding fails
            Some(k) => PublicKey::try_from(k).ok(),
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
