#![warn(unused_qualifications)]
#![allow(unused_imports)]
// Clippy broken; doesn't recognize certain imports are used and sees them as unused

use crate::{
    account::AccountInfo,
    chain::config::ChainClientConfig,
    cosmos_modules,
    error::{AirdropError, ChainClientError},
    keyring::Keyring,
    tx::{Coin, MultiSendIo, Payment, PaymentsToml, TxMetadata},
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
    ) -> Result<(), ChainClientError> {
        // Verify grant exists for grantee from granter for MsgSend
        let res = self
            .query_authz_grant(granter.as_ref(), grantee.as_ref(), MSG_MULTI_SEND_URL)
            .await?;

        // If any grants meet the following criteria we can be confident the transaction is authorized:
        // 1. The grant either has no expiration, or an expiration with more than 60 seconds remaining.
        // 2. The grant contains a generic authorization
        let grant_found = res.grants.iter().any(|g| {
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
            // I don't actually think this is a necessary check as there is no way to specify
            // authorization for MultiSend without using a generic one
            let authorization = g.authorization.clone().unwrap();
            if authorization.type_url.as_str() != GENERIC_AUTHORIZATION_URL {
                return false;
            }

            true
        });
        if !grant_found {
            return Err(ChainClientError::UnauthorizedTx(format!(
                "no relevant grant exists for {} on behalf of {}",
                grantee.as_ref(),
                granter.as_ref()
            )));
        }

        Ok(())
    }

    pub async fn execute_delegated_airdrop(
        &mut self,
        granter: AccountInfo,
        grantee: AccountInfo,
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

        self.execute_authorized_tx(grantee, msgs, tx_metadata).await
    }

    pub async fn execute_delegated_airdrop_from_toml(
        &mut self,
        path: String,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let payments_toml = read_payments_toml(path)?;
        let grantee = match payments_toml.grantee_key_name {
            Some(g) => self.keyring.get_account(&g, &self.config.account_prefix),
            None => return Err(AirdropError::Toml("no grantee key name was provided for the delegated airdrop.".to_string()).into()),
        }?;
        let granter = self.keyring.get_account(&payments_toml.sender_key_name, &self.config.account_prefix)?;

        // add fee_payer and fee_granter values to metadata if present
        let basic_tx_metadata = self.get_basic_tx_metadata().await?;
        let mut tx_metadata = tx_metadata.unwrap_or(basic_tx_metadata);
        let tx_metadata = match (payments_toml.fee_granter, payments_toml.fee_payer) {
            (None, Some(fp)) => {
                tx_metadata.fee_payer = Some(AccountId::from_str(&fp)?);
                tx_metadata
            }
            (Some(fg), None) =>  {
                tx_metadata.fee_granter = Some(AccountId::from_str(&fg)?);
                tx_metadata
            },
            (Some(fg), Some(fp)) => {
                tx_metadata.fee_payer = Some(AccountId::from_str(&fp)?);
                tx_metadata.fee_granter = Some(AccountId::from_str(&fg)?);
                tx_metadata
            }
            _ => tx_metadata
        };

        self.execute_delegated_airdrop(granter, grantee, payments_toml.payments, Some(tx_metadata))
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
        path: String,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let payments_toml = read_payments_toml(path)?;
        let sender = self.keyring.get_account(&payments_toml.sender_key_name, &self.config.account_prefix)?;
        self.execute_airdrop(sender, payments_toml.payments, tx_metadata).await
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
pub fn read_payments_toml(path: String) -> Result<PaymentsToml, ChainClientError> {
    let toml_string = fs::read_to_string(path)?;
    Ok(toml::from_str(toml_string.as_str())?)
}

pub fn write_payments_toml(
    path: String,
    sender_key_name: String,
    grantee_key_name: Option<String>,
    fee_payer: Option<AccountId>,
    fee_granter: Option<AccountId>,
    payments: Vec<Payment>,
) -> Result<(), ChainClientError> {
    let fee_payer = fee_payer.map_or(None, |fp| Some(fp.as_ref().to_string()));
    let fee_granter = fee_granter.map_or(None, |fg| Some(fg.as_ref().to_string()));
    let toml_obj = PaymentsToml {
        sender_key_name,
        grantee_key_name,
        fee_granter,
        fee_payer,
        payments: payments,
    };
    let toml_string = toml::to_string(&toml_obj)?;
    Ok(fs::write(path, toml_string)?)
}

#[cfg(test)]
mod tests {
    use std::{fs::Permissions, os::unix::prelude::PermissionsExt};

    use super::*;
    use assay::assay;

    #[assay]
    fn writes_and_reads_payments_toml() {
        // Set up payments and file path
        let payment1 = Payment {
            recipient: "bob".to_string(),
            amount: 100,
            denom: "dollarbucks".to_string(),
        };
        let payment2 = Payment {
            recipient: "alice".to_string(),
            amount: 35,
            denom: "dingos".to_string(),
        };
        let payment3 = Payment {
            recipient: "frank".to_string(),
            amount: 10,
            denom: "dollarbucks".to_string(),
        };
        let payments = vec![payment1, payment2, payment3];
        let path_string = std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/payments_toml_test";
        fs::create_dir_all(&path_string)?;
        #[cfg(unix)]
        fs::set_permissions(&path_string, Permissions::from_mode(0o700))?;
        let path = Path::new(&path_string).canonicalize()?;
        let st = path.metadata()?;

        assert!(st.is_dir());

        #[cfg(unix)]
        assert!(st.permissions().mode() & 0o777 == 0o700);

        let sender_key = "sender_key".to_string();
        let grantee_key = Some("grantee_key".to_string());
        let fee_granter =
            Some(AccountId::from_str("cosmos142nrqssptljjajdkav8djftp87lvg0ghvm0m9c").unwrap());
        let fee_payer =
            Some(AccountId::from_str("cosmos1svs56wmqsezpjqgmvaf78rx3ut94pw6s7mxl05").unwrap());
        let expected_result = PaymentsToml {
            sender_key_name: sender_key.clone(),
            grantee_key_name: grantee_key.clone(),
            fee_granter: Some(fee_granter.clone().unwrap().as_ref().to_string()),
            fee_payer: Some(fee_payer.clone().unwrap().as_ref().to_string()),
            payments: payments.clone(),
        };

        // Write and read payments toml
        let file_path = path_string.clone() + "payments.toml";
        write_payments_toml(
            file_path.clone(),
            sender_key,
            grantee_key,
            fee_payer,
            fee_granter,
            payments.clone(),
        )
        .expect("failed to write payments toml");

        let result = read_payments_toml(file_path).expect("failed to read payments toml");

        assert_eq!(result, expected_result);

        // Clean up dir
        std::fs::remove_dir_all(path).expect(&format!(
            "Failed to delete test directory {:?}",
            path_string.clone()
        ));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(path_string).unwrap());
        assert!(result.is_err());
    }
}
