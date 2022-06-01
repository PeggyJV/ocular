use crate::{
    account::{Account, BaseAccount},
    cosmos_modules::{
        authz::{self, *},
        feegrant::{BasicAllowance, MsgGrantAllowance},
    },
    error::{ChainClientError, GrpcError},
    tx::TxMetadata,
};
use cosmrs::{tx, AccountId};
use prost::Message;
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
use tonic::transport::Channel;

use super::ChainClient;

pub type AuthzQueryClient = authz::query_client::QueryClient<Channel>;

impl ChainClient {
    // Authz queries
    pub async fn get_authz_query_client(&self) -> Result<AuthzQueryClient, ChainClientError> {
        self.check_for_grpc_address()?;

        AuthzQueryClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(|e| GrpcError::Connection(e).into())
    }

    // Query for a specific msg grant
    pub async fn query_authz_grant(
        &self,
        granter: &str,
        grantee: &str,
        msg_type_url: &str,
    ) -> Result<QueryGrantsResponse, ChainClientError> {
        let mut query_client = self.get_authz_query_client().await?;

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

    // Grant Authorization
    // TODO: support other types of authorization grants other than GenericAuthorization for send messages.
    pub async fn grant_send_authorization(
        &self,
        granter: Account,
        grantee: AccountId,
        expiration_timestamp: Option<prost_types::Timestamp>,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let signer = granter.private_key;
        let msg = MsgGrant {
            granter: granter.id.to_string(),
            grantee: grantee.to_string(),
            grant: Some(Grant {
                authorization: Some(prost_types::Any {
                    type_url: String::from("/cosmos.authz.v1beta1.GenericAuthorization"),
                    value: GenericAuthorization {
                        msg: String::from("/cosmos.bank.v1beta1.MsgSend"),
                    }
                    .encode_to_vec(),
                }),
                expiration: expiration_timestamp,
            }),
        };
        let granter = self.query_account(granter.id.as_ref().to_string()).await?;
        let granter = BaseAccount::try_from(granter)?;

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            granter,
            signer,
            tx_body,
            tx_metadata,
            None,
            None,
        )
        .await
    }

    // Revoke Authorization
    // TODO: support other types of authorization revokes other than send messages.
    pub async fn revoke_send_authorization(
        &self,
        granter: Account,
        grantee: AccountId,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let signer = granter.private_key;
        let msg = MsgRevoke {
            granter: granter.id.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
        };
        let granter = self.query_account(granter.id.as_ref().to_string()).await?;
        let granter = BaseAccount::try_from(granter)?;

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            granter,
            signer,
            tx_body,
            tx_metadata,
            None,
            None,
        )
        .await
    }

    // Execute a transaction previously authorized by granter
    pub async fn execute_authorized_tx(
        &self,
        grantee: Account,
        msgs: Vec<::prost_types::Any>,
        tx_metadata: TxMetadata,
        fee_payer: Option<AccountId>,
        fee_granter: Option<AccountId>,
    ) -> Result<Response, ChainClientError> {
        let signer = grantee.private_key;
        let msg = MsgExec {
            grantee: grantee.id.to_string(),
            msgs,
        };
        let grantee = self.query_account(grantee.id.as_ref().to_string()).await?;
        let grantee = BaseAccount::try_from(grantee)?;

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
            value: msg.encode_to_vec(),
        };

        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            grantee,
            signer,
            tx_body,
            tx_metadata,
            fee_payer,
            fee_granter,
        )
        .await
    }

    // Basic fee allowance
    pub async fn perform_basic_allowance_fee_grant(
        &self,
        granter: Account,
        grantee: AccountId,
        expiration: Option<prost_types::Timestamp>,
        // TODO: Standardize below Coin type to common cosmrs coin type once FeeGrants get looped in.
        spend_limit: cosmos_sdk_proto::cosmos::base::v1beta1::Coin,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let signer = granter.private_key;
        let allowance = BasicAllowance {
            spend_limit: vec![spend_limit],
            expiration,
        };
        let msg = MsgGrantAllowance {
            granter: granter.id.to_string(),
            grantee: grantee.to_string(),
            allowance: Some(prost_types::Any {
                type_url: String::from("/cosmos.feegrant.v1beta1.BasicAllowance"),
                value: allowance.encode_to_vec(),
            }),
        };
        let granter = self.query_account(granter.id.as_ref().to_string()).await?;
        let granter = BaseAccount::try_from(granter)?;

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.feegrant.v1beta1.MsgGrantAllowance"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            granter,
            signer,
            tx_body,
            tx_metadata,
            None,
            None,
        )
        .await
    }
}

// Disclaimer on testing: Since the above commands inherently require chains to operate, testing is primarily deferred to integration tests in ocular/tests/single_node_chain_txs.rs

#[cfg(test)]
mod tests {
    use crate::chain::{self, client::ChainClient};
    use assay::assay;

    #[assay]
    async fn gets_authz_client() {
        let client = ChainClient::new(chain::COSMOSHUB).unwrap();

        client
            .get_authz_query_client()
            .await
            .expect("failed to get authz query client");
    }
}
