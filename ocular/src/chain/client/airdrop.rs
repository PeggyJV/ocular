#![warn(unused_qualifications)]
#![allow(unused_imports)]
// Clippy broken; doesn't recognize certain imports are used and sees them as unused

use crate::{
    account::AccountInfo,
    chain::config::ChainClientConfig,
    cosmos_modules,
    error::{AutomatedTxHandlerError, ChainClientError},
    keyring::Keyring,
    tx::{Coin, MultiSendIo, Payment, TxMetadata},
};
use bip32::Mnemonic;
use cosmos_sdk_proto::cosmos::authz::v1beta1::Grant;
use cosmrs::{
    bank::{MsgMultiSend, MsgSend},
    rpc,
    tx::Msg,
    AccountId,
};

use prost::Message;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};
use std::{fs, path::Path, str::FromStr};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
use uuid::Uuid;

use super::ChainClient;

const MSG_MULTI_SEND_URL: &str = "/cosmos.bank.v1beta1.MsgMultiSend";
const GENERIC_AUTHORIZATION_URL: &str = "/cosmos.authz.v1beta1.GenericAuthorization";

impl ChainClient {
    pub async fn verify_multi_send_grant(
        &mut self,
        granter: AccountId,
        grantee: AccountId,
    ) -> Result<bool, ChainClientError> {
        // Verify grant exists for grantee from granter for MsgSend
        let res = self
            .query_authz_grant(granter.as_ref(), grantee.as_ref(), MSG_MULTI_SEND_URL)
            .await?;

        // If any grants meet the following criteria we can be confident the transaction is authorized:
        // 1. The grant either has no expiration, or an expiration with more than 60 seconds remaining.
        // 2. The grant contains a generic authorization
        Ok(res.grants.iter().any(|g| {
            if g.expiration.is_some() {
                let expiration = g.expiration.clone().unwrap();
                let cutoff = i64::try_from(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + 60,
                )
                .expect("failed to derive time from system clock");

                if expiration.seconds <= cutoff {
                    return false;
                }
            }
            if g.authorization.is_none() {
                return false;
            }
            // I don't actually thing this is a necessary check as there is no way to specify
            // authorization for MultiSend without using a generic one
            let authorization = g.authorization.clone().unwrap();
            if authorization.type_url.as_str() != GENERIC_AUTHORIZATION_URL {
                return false;
            }

            true
        }))
    }

    pub async fn execute_delegated_airdrop(
        &mut self,
        granter: AccountInfo,
        grantee: AccountInfo,
        fee_payer: Option<AccountId>,
        fee_granter: Option<AccountId>,
        payments: Vec<Payment>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        self.verify_multi_send_grant(granter.id.clone(), grantee.id.clone())
            .await?;
        let (inputs, outputs) =
            multi_send_args_from_payments(granter.id.as_ref().to_string(), payments);
        let msgs: Vec<prost_types::Any> = vec![MsgMultiSend {
            inputs: inputs
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            outputs: outputs
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        }
        .to_any()?];

        self.execute_authorized_tx(grantee, fee_payer, fee_granter, msgs, tx_metadata)
            .await
    }

    pub async fn execute_delegated_airdrop_from_toml(
        &mut self,
        granter: AccountInfo,
        grantee: AccountInfo,
        fee_payer: Option<AccountId>,
        fee_granter: Option<AccountId>,
        path: String,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let payments = read_payments_toml(path)?;
        self.execute_delegated_airdrop(
            granter,
            grantee,
            fee_payer,
            fee_granter,
            payments,
            tx_metadata,
        )
        .await
    }

    pub async fn execute_airdrop(
        &mut self,
        sender: AccountInfo,
        payments: Vec<Payment>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let (inputs, outputs) =
            multi_send_args_from_payments(sender.id.as_ref().to_string(), payments);
        self.multi_send(sender, inputs, outputs, tx_metadata).await
    }

    pub async fn execute_airdrop_from_toml(
        &mut self,
        sender: AccountInfo,
        path: String,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let payments = read_payments_toml(path)?;
        self.execute_airdrop(sender, payments, tx_metadata).await
    }
}

pub fn multi_send_args_from_payments(
    sender_addr: String,
    payments: Vec<Payment>,
) -> (Vec<MultiSendIo>, Vec<MultiSendIo>) {
    let mut inputs = Vec::<MultiSendIo>::new();
    let mut outputs = inputs.clone();
    payments.iter().for_each(|p| {
        let coins = vec![Coin {
            denom: p.denom.clone(),
            amount: p.amount,
        }];
        inputs.push(MultiSendIo {
            address: sender_addr.clone(),
            coins: coins.clone(),
        });
        outputs.push(MultiSendIo {
            address: p.recipient.clone(),
            coins,
        });
    });
    (inputs, outputs)
}

// TO-DO different error type.
pub fn read_payments_toml(path: String) -> Result<Vec<Payment>, ChainClientError> {
    let toml_string = fs::read_to_string(path)?;
    Ok(toml::from_str(toml_string.as_str())?)
}

pub fn write_payments_toml(payments: &[Payment], path: String) -> Result<(), ChainClientError> {
    let toml_string = toml::to_string(payments)?;
    Ok(fs::write(path, toml_string)?)
}
