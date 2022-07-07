use crate::{
    account::AccountInfo,
    cosmos_modules::{
        authz::*,
        feegrant::{BasicAllowance, MsgGrantAllowance},
    },
    error::ChainClientError,
    tx::TxMetadata,
};
use cosmrs::{tx, AccountId};
use prost::Message;
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use super::ChainClient;

impl ChainClient {
    // Grant Authorization
    // TODO: support other types of authorization grants other than GenericAuthorization for send messages.
    pub async fn grant_send_authorization(
        &mut self,
        granter: AccountInfo,
        grantee: AccountId,
        expiration_timestamp: Option<prost_types::Timestamp>,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgGrant {
            granter: granter.address(&self.config.account_prefix)?,
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
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, tx_body, tx_metadata).await
    }

    // Revoke Authorization
    // TODO: support other types of authorization revokes other than send messages.
    pub async fn revoke_send_authorization(
        &mut self,
        granter: AccountInfo,
        grantee: AccountId,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgRevoke {
            granter: granter.address(&self.config.account_prefix)?,
            grantee: grantee.to_string(),
            msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
        };
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, tx_body, tx_metadata).await
    }

    // Execute a transaction previously authorized by another account on its behalf
    pub async fn execute_authorized_tx(
        &mut self,
        grantee: AccountInfo,
        msgs: Vec<::prost_types::Any>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgExec {
            grantee: grantee.address(&self.config.account_prefix)?,
            msgs,
        };
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
            value: msg.encode_to_vec(),
        };
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(grantee, tx_body, tx_metadata).await
    }

    // Basic fee allowance
    pub async fn perform_basic_allowance_fee_grant(
        &mut self,
        granter: AccountInfo,
        grantee: AccountId,
        expiration: Option<prost_types::Timestamp>,
        // TODO: Standardize below Coin type to common cosmrs coin type once FeeGrants get looped in.
        spend_limit: cosmos_sdk_proto::cosmos::base::v1beta1::Coin,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let allowance = BasicAllowance {
            spend_limit: vec![spend_limit],
            expiration,
        };
        let msg = MsgGrantAllowance {
            granter: granter.address(&self.config.account_prefix)?,
            grantee: grantee.to_string(),
            allowance: Some(prost_types::Any {
                type_url: String::from("/cosmos.feegrant.v1beta1.BasicAllowance"),
                value: allowance.encode_to_vec(),
            }),
        };
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.feegrant.v1beta1.MsgGrantAllowance"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, tx_body, tx_metadata).await
    }
}
