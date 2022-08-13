#![warn(unused_qualifications)]
#![allow(unused_imports)]
// Clippy broken; doesn't recognize certain imports are used and sees them as unused

use crate::{
    account::AccountInfo,
    chain::config::ChainClientConfig,
    cosmos_modules,
    error::{AirdropError, ChainClientError},
    keyring::Keyring,
    tx::{Any, Coin, MultiSendIo, Payment, PaymentsToml, TxMetadata},
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
use uuid::Uuid;

use super::{tx::BroadcastCommitResponse, ChainClient};

const MSG_MULTI_SEND_URL: &str = "/cosmos.bank.v1beta1.MsgMultiSend";
const GENERIC_AUTHORIZATION_URL: &str = "/cosmos.authz.v1beta1.GenericAuthorization";

impl ChainClient {
    pub async fn verify_multi_send_grant(
        &mut self,
        granter: &AccountId,
        grantee: &AccountId,
    ) -> Result<(), ChainClientError> {
        // Verify grant exists for grantee from granter for MsgSend
        let grants = self
            .query_authz_grant(granter.as_ref(), grantee.as_ref(), MSG_MULTI_SEND_URL)
            .await?;

        // If any grants meet the following criteria we can be confident the transaction is authorized:
        // 1. The grant either has no expiration, or an expiration with more than 60 seconds remaining.
        // 2. The grant contains a generic authorization
        let grant_found = grants.iter().any(|g| {
            if g.expiration.is_none() {
                return false;
            }

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
        granter: &AccountId,
        grantee: &AccountInfo,
        payments: Vec<Payment>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        self.verify_multi_send_grant(granter, &grantee.id(&self.config.account_prefix)?)
            .await?;

        let (inputs, outputs) = multi_send_args_from_payments(&granter.to_string(), payments);
        let msgs: Vec<Any> = vec![MsgMultiSend {
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
        path: &str,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let payments_toml = read_payments_toml(path)?;
        let grantee = match payments_toml.grantee_key_name {
            Some(g) => self.keyring.get_account(&g),
            None => {
                return Err(AirdropError::Toml(
                    "no grantee key name was provided for the delegated airdrop.".to_string(),
                )
                .into())
            }
        }?;
        let granter = self.keyring.get_account(&payments_toml.sender_key_name)?;

        // add fee_payer and fee_granter values to metadata if present
        let basic_tx_metadata = self.get_basic_tx_metadata().await?;
        let tx_metadata = tx_metadata.unwrap_or(basic_tx_metadata);

        self.execute_delegated_airdrop(
            &granter.id(&self.config.account_prefix)?,
            &grantee,
            payments_toml.payments,
            Some(tx_metadata),
        )
        .await
    }

    pub async fn execute_airdrop(
        &mut self,
        sender: &AccountInfo,
        payments: Vec<Payment>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let (inputs, outputs) =
            multi_send_args_from_payments(&sender.address(&self.config.account_prefix)?, payments);
        self.multi_send(sender, inputs, outputs, tx_metadata).await
    }

    pub async fn execute_airdrop_from_toml(
        &mut self,
        path: &str,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<BroadcastCommitResponse, ChainClientError> {
        let payments_toml = read_payments_toml(path)?;
        let sender = self.keyring.get_account(&payments_toml.sender_key_name)?;
        // TO-DO user the metadata from the toml
        self.execute_airdrop(&sender, payments_toml.payments, tx_metadata)
            .await
    }
}

/// Creates arguments for a MultiSend transaction from a vec of [`Payment`]. We require a single
/// `Input` because, for Authz transactions, the tx will be considered to have multiple signers if
/// there are multiple `Input`s, which is not allowed by the authz msg `MsgExec`.
pub fn multi_send_args_from_payments(
    sender_addr: &str,
    payments: Vec<Payment>,
) -> (Vec<MultiSendIo>, Vec<MultiSendIo>) {
    let mut outputs = Vec::<MultiSendIo>::new();
    let mut coins_total = HashMap::<String, u64>::new();
    payments.iter().for_each(|p| {
        let key = p.denom.clone();
        let value = p.amount;
        if coins_total.contains_key(&key) {
            coins_total.insert(key.clone(), coins_total.get(&key).unwrap() + value);
        } else {
            coins_total.insert(key, value);
        }

        outputs.push(MultiSendIo {
            address: p.recipient.clone(),
            coins: vec![Coin {
                denom: p.denom.clone(),
                amount: p.amount,
            }],
        });
    });

    let coins_input = coins_total
        .iter()
        .map(|kv| Coin {
            denom: kv.0.to_owned(),
            amount: *kv.1,
        })
        .collect();
    let input = vec![MultiSendIo {
        address: sender_addr.to_string(),
        coins: coins_input,
    }];
    (input, outputs)
}

// TO-DO different error type.
pub fn read_payments_toml(path: &str) -> Result<PaymentsToml, ChainClientError> {
    let toml_string = fs::read_to_string(path)?;
    Ok(toml::from_str(toml_string.as_str())?)
}

pub fn write_payments_toml(
    path: &str,
    sender_key_name: &str,
    grantee_key_name: Option<&str>,
    payments: Vec<Payment>,
) -> Result<(), ChainClientError> {
    let grantee_key_name = grantee_key_name.map(|v| v.to_string());
    let toml_obj = PaymentsToml {
        sender_key_name: sender_key_name.to_string(),
        grantee_key_name,
        payments,
    };
    let toml_string = toml::to_string(&toml_obj)?;
    Ok(fs::write(path, toml_string)?)
}

#[cfg(test)]
mod tests {
    use std::{fs::Permissions, os::unix::prelude::PermissionsExt};

    use super::*;
    use assay::assay;
    use rand::Rng;

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
        let grantee_key = Some("grantee_key");
        let expected_result = PaymentsToml {
            sender_key_name: sender_key.clone(),
            grantee_key_name: grantee_key.map(|v| v.to_string()),
            payments: payments.clone(),
        };

        // Write and read payments toml
        let file_path = path_string.clone() + "payments.toml";
        write_payments_toml(
            &file_path.clone(),
            &sender_key,
            grantee_key,
            payments.clone(),
        )
        .expect("failed to write payments toml");

        let result = read_payments_toml(&file_path).expect("failed to read payments toml");

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

    #[assay]
    fn builds_multisend_args() {
        let prefix = "test";
        let sender = AccountInfo::new("");
        let sender_address = sender.address(prefix).unwrap();
        let mut rng = rand::thread_rng();
        let num_outputs = rng.gen_range(1..25);
        let accounts = generate_accounts(num_outputs);
        let payments = generate_payments_single_denom(prefix, "utest", &accounts);
        let args = multi_send_args_from_payments(&sender_address.clone(), payments);
        let input_total: u64 = args
            .0
            .iter()
            .map(|io: &MultiSendIo| io.coins[0].amount)
            .sum();
        let output_total: u64 = args
            .1
            .iter()
            .map(|io: &MultiSendIo| io.coins[0].amount)
            .sum();

        assert_eq!(args.0.len(), 1);
        assert_eq!(args.1.len() as u64, num_outputs);

        let mut addresses: Vec<String> = accounts
            .iter()
            .map(|acc| acc.address(prefix).unwrap())
            .collect();
        addresses.sort();
        let mut args_out_addresses: Vec<String> = args
            .1
            .iter()
            .map(|io: &MultiSendIo| io.address.clone())
            .collect();
        args_out_addresses.sort();

        assert_eq!(addresses, args_out_addresses);
        assert_eq!(args.0[0].address, sender_address);
        assert_eq!(input_total, output_total);
    }

    fn generate_accounts(n: u64) -> Vec<AccountInfo> {
        let mut accounts = Vec::<AccountInfo>::new();

        for _ in 0..n {
            accounts.push(AccountInfo::new(""));
        }

        accounts
    }

    fn generate_payments_single_denom(
        prefix: &str,
        denom: &str,
        accounts: &Vec<AccountInfo>,
    ) -> Vec<Payment> {
        let mut rng = rand::thread_rng();
        accounts
            .iter()
            .map(|a| Payment {
                recipient: a.address(prefix).unwrap(),
                amount: rng.gen_range(1..99999),
                denom: denom.to_string(),
            })
            .collect()
    }
}
