//! Defines methods for Authz module Msgs
use std::time::{Duration, SystemTime};

use crate::{
    account::AccountInfo,
    cosmos_modules::{
        authz::*,
        feegrant::{BasicAllowance, MsgGrantAllowance},
    },
    error::ChainClientError,
    tx::{Any, TxMetadata},
    Timestamp,
};
use cosmrs::{tx, AccountId};
use prost::Message;

use super::{BroadcastCommitResponse, ChainClient};

impl ChainClient {
    /// Grant a generic authorization for the provided Msg and expiration
    // TODO: support other types of authorization grants other than GenericAuthorization for send messages.
    pub async fn grant_generic_authorization(
        &mut self,
        granter: &AccountInfo,
        grantee: AccountId,
        message: &str,
        expiration_timestamp: Option<Timestamp>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let expiration: Timestamp = match expiration_timestamp {
            Some(exp) => exp,
            None => {
                // defaults to 24 hour expiration
                let timestamp = SystemTime::now()
                    .checked_add(Duration::from_secs(31536000))
                    .unwrap();
                Timestamp::from(timestamp)
            }
        };

        let msg = MsgGrant {
            granter: granter.address(&self.config.account_prefix)?,
            grantee: grantee.to_string(),
            grant: Some(Grant {
                authorization: Some(Any {
                    type_url: String::from("/cosmos.authz.v1beta1.GenericAuthorization"),
                    value: GenericAuthorization {
                        msg: String::from(message),
                    }
                    .encode_to_vec(),
                }),
                expiration: Some(expiration),
            }),
        };
        let msg_any = Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
            value: msg.encode_to_vec(),
        };
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, tx_body, tx_metadata).await
    }

    /// Revoke authorization for `MsgSend`
    // TODO: support other types of authorization revokes other than send messages.
    pub async fn revoke_send_authorization(
        &mut self,
        granter: &AccountInfo,
        grantee: AccountId,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let msg = MsgRevoke {
            granter: granter.address(&self.config.account_prefix)?,
            grantee: grantee.to_string(),
            msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
        };
        let msg_any = Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
            value: msg.encode_to_vec(),
        };
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, tx_body, tx_metadata).await
    }

    /// Execute a transaction previously authorized by another account on its behalf
    pub async fn execute_authorized_tx(
        &mut self,
        grantee: &AccountInfo,
        msgs: Vec<Any>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let msg = MsgExec {
            grantee: grantee.address(&self.config.account_prefix)?,
            msgs,
        };
        let msg_any = Any {
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

    /// Basic fee allowance
    pub async fn perform_basic_allowance_fee_grant(
        &mut self,
        granter: &AccountInfo,
        grantee: AccountId,
        expiration: Option<Timestamp>,
        // TODO: Standardize below Coin type to common cosmrs coin type once FeeGrants get looped in.
        spend_limit: cosmos_sdk_proto::cosmos::base::v1beta1::Coin,
        tx_metadata: TxMetadata,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let allowance = BasicAllowance {
            spend_limit: vec![spend_limit],
            expiration,
        };
        let msg = MsgGrantAllowance {
            granter: granter.address(&self.config.account_prefix)?,
            grantee: grantee.to_string(),
            allowance: Some(Any {
                type_url: String::from("/cosmos.feegrant.v1beta1.BasicAllowance"),
                value: allowance.encode_to_vec(),
            }),
        };
        let msg_any = Any {
            type_url: String::from("/cosmos.feegrant.v1beta1.MsgGrantAllowance"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, tx_body, tx_metadata).await
    }
}
