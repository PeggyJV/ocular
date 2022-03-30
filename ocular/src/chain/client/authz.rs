use crate::{
    chain::client::tx::{Account, TxMetadata},
    cosmos_modules::*,
    error::TxError,
};
use cosmos_sdk_proto::cosmos::authz::v1beta1::{
    GenericAuthorization, MsgExec, MsgGrant, MsgRevoke,
};
use prost::Message;
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use cosmrs::{tx, Coin};

use super::ChainClient;

impl ChainClient {
    // Grant Authorization
    // TODO: support other types of authorization grants other than GenericAuthorization for send messages.
    pub async fn grant_send_authorization(
        &self,
        granter: Account,
        grantee: Account,
        expiration_timestamp: Option<prost_types::Timestamp>,
        tx_metadata: TxMetadata,
        denom: &str,
    ) -> Result<Response, TxError> {
        let msg = MsgGrant {
            granter: granter.id.to_string(),
            grantee: grantee.id.to_string(),
            grant: Some(authz::Grant {
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

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
            value: msg.encode_to_vec(),
        };

        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            granter.public_key,
            granter.private_key,
            Coin {
                amount: 0u8.into(),
                denom: denom.parse().expect("Could not parse denom."),
            },
            tx_body,
            tx_metadata,
        )
        .await
    }

    // Revoke Authorization
    // TODO: support other types of authorization revokes other than send messages.
    pub async fn revoke_send_authorization(
        &self,
        granter: Account,
        grantee: Account,
        tx_metadata: TxMetadata,
        denom: &str,
    ) -> Result<Response, TxError> {
        let msg = MsgRevoke {
            granter: granter.id.to_string(),
            grantee: grantee.id.to_string(),
            msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
        };

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
            value: msg.encode_to_vec(),
        };

        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            granter.public_key,
            granter.private_key,
            Coin {
                amount: 0u8.into(),
                denom: denom.parse().expect("Could not parse denom."),
            },
            tx_body,
            tx_metadata,
        )
        .await
    }

    // Execute a transaction previously authorized by granter
    pub async fn execute_authorized_tx(
        &self,
        grantee: Account,
        msgs: Vec<::prost_types::Any>,
        tx_metadata: TxMetadata,
        denom: &str,
    ) -> Result<Response, TxError> {
        let msg = MsgExec {
            grantee: grantee.id.to_string(),
            msgs,
        };

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
            value: msg.encode_to_vec(),
        };

        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(
            grantee.public_key,
            grantee.private_key,
            Coin {
                amount: 0u8.into(),
                denom: denom.parse().expect("Could not parse denom."),
            },
            tx_body,
            tx_metadata,
        )
        .await
    }
}

// Disclaimer on testing: Since the above commands inherently require chains to operate, testing is primarily deferred to integration tests in ocular/tests/single_node_chain_txs.rs
