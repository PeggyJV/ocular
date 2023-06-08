//! Queries and messages for the [FeeGrant module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/feegrant/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`FeeGrantQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{
    feegrant::{MsgGrantAllowance, MsgRevokeAllowance},
    tx::Msg,
    AccountId, Any,
};
use eyre::{Context, Report, Result};
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::feegrant::v1beta1::{self as feegrant, QueryAllowancesResponse},
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The gov module's query client proto definition
pub type FeeGrantQueryClient = feegrant::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<FeeGrantQueryClient> for FeeGrantQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        FeeGrantQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Allowance returns fee granted to the grantee by the granter.
    pub async fn query_allowance(
        &mut self,
        granter: &str,
        grantee: &str,
    ) -> Result<feegrant::QueryAllowanceResponse> {
        let query_client = self.get_client::<FeeGrantQueryClient>().await?;
        let request = feegrant::QueryAllowanceRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
        };

        Ok(query_client.allowance(request).await?.into_inner())
    }

    /// Allowances returns all the grants for address.
    pub async fn query_all_allowances(
        &mut self,
        grantee: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryAllowancesResponse> {
        let query_client = self.get_client::<FeeGrantQueryClient>().await?;
        let request = feegrant::QueryAllowancesRequest {
            grantee: grantee.to_string(),
            pagination,
        };

        Ok(query_client.allowances(request).await?.into_inner())
    }
}

/// Represents a [FeeGrant module message](https://docs.cosmos.network/v0.45/modules/feegrant/03_messages.html)
#[derive(Clone, Debug)]
pub enum FeeGrant<'m> {
    /// Grant an allowance of funds to another account to cover transaction fees. Represents a [`MsgGrantAllowance`]
    GrantAllowance {
        /// Address of the granting account
        granter: &'m str,
        /// Address of the account being granted the allowance
        grantee: &'m str,
        /// Allowance to be granted
        allowance: Any,
    },
    /// Revoke a previously granted allowance. Represents a [`MsgRevokeAllowance`]
    RevokeAllowance {
        /// Address of the granting account
        granter: &'m str,
        /// Address of the grantee
        grantee: &'m str,
    },
}

impl ModuleMsg for FeeGrant<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            FeeGrant::GrantAllowance {
                granter,
                grantee,
                allowance,
            } => MsgGrantAllowance {
                granter: AccountId::from_str(granter)?,
                grantee: AccountId::from_str(grantee)?,
                allowance: Some(allowance),
            }
            .to_any(),
            FeeGrant::RevokeAllowance { granter, grantee } => MsgRevokeAllowance {
                granter: AccountId::from_str(granter)?,
                grantee: AccountId::from_str(grantee)?,
            }
            .to_any(),
        }
    }

    /// Converts the message enum representation into an [`UnsignedTx`] containing the corresponding Msg
    fn into_tx(self) -> Result<UnsignedTx> {
        let mut tx = UnsignedTx::new();
        tx.add_msg(self.into_any()?);

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        FeeGrant::GrantAllowance {
            granter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            grantee: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            allowance: Any::default(),
        }
        .into_tx()
        .unwrap();

        FeeGrant::RevokeAllowance {
            granter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            grantee: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();
    }
}
