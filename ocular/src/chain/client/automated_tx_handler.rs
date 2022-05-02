#![warn(unused_qualifications)]
#![allow(unused_imports)]
// Clippy broken; doesn't recognize certain imports are used and sees them as unused

use crate::{
    chain::{
        client::tx::{Account, TxMetadata},
        config::ChainClientConfig,
    },
    cosmos_modules,
    error::AutomatedTxHandlerError,
    keyring::Keyring,
};
use bip32::Mnemonic;
use cosmrs::{bank::MsgSend, rpc, tx::Msg, AccountId, Coin};

use prost::Message;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{fs, path::Path, str::FromStr};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
use uuid::Uuid;

use super::ChainClient;

const MSG_SEND_URL: &str = "/cosmos.bank.v1beta1.MsgSend";
const GENERIC_AUTHORIZATION_URL: &str = "/cosmos.authz.v1beta1.GenericAuthorization";
const SEND_AUTHORIZATION_URL: &str = "/cosmos.bank.v1beta1.SendAuthorization";

// Toml structs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DelegatedToml<'a> {
    pub sender: DelegatedSender<'a>,

    #[serde(borrow)]
    pub transaction: Vec<DelegateTransaction<'a>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DelegatedSender<'a> {
    pub grantee_private_key_path: &'a str,
    pub granter_account: &'a str,
    pub denom: &'a str,
    // MsgExec data
    pub gas_fee: u64,
    pub gas_limit: u64,
    pub timeout_height: u32,
    pub memo: &'a str,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DelegateTransaction<'a> {
    pub name: &'a str,
    pub destination_account: &'a str,
    pub amount: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BatchToml<'a> {
    pub sender: BatchSender<'a>,
    #[serde(borrow)]
    pub transactions: Vec<BatchTransaction<'a>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BatchSender<'a> {
    pub source_private_key_path: &'a str,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BatchTransaction<'a> {
    pub name: &'a str,
    pub destination_account: &'a str,
    pub amount: u64,
    pub denom: &'a str,
    pub gas_limit: u64,
    pub gas_fee: u64,
    pub timeout_height: u32,
    pub memo: &'a str,
}

impl ChainClient {
    // Creates a brand new temporary key for delegated tx workflows.
    pub async fn execute_delegated_transacton_toml(
        &mut self,
        toml_path: String,
        verify_msg_grant: bool,
    ) -> Result<Response, AutomatedTxHandlerError> {
        let content = match fs::read_to_string(toml_path) {
            Ok(result) => result,
            Err(err) => {
                return Err(AutomatedTxHandlerError::FileIO(err.to_string()));
            }
        };

        let toml: DelegatedToml = match toml::from_str(&content) {
            Ok(result) => result,
            Err(err) => {
                return Err(AutomatedTxHandlerError::Toml(err.to_string()));
            }
        };

        dbg!(&toml);

        // Convert granter info into cosmrs types
        let granter_account_id = match AccountId::from_str(toml.sender.granter_account) {
            Ok(res) => res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyHandling(err.to_string())),
        };

        // Add grantee to keyring & parse out relevant types
        let grantee_key_name = &(String::from("grantee") + &Uuid::new_v4().to_string());

        match self.keyring.add_key_from_path(
            grantee_key_name,
            toml.sender.grantee_private_key_path,
            false,
        ) {
            Ok(_res) => _res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyStore(err.to_string())),
        };

        let grantee_public_info = match self
            .keyring
            .get_public_key_and_address(grantee_key_name, &self.config.account_prefix)
        {
            Ok(res) => res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyStore(err.to_string())),
        };

        if verify_msg_grant {
            // Verify grant exists for grantee from granter for MsgSend
            match self
                .query_authz_grant(
                    granter_account_id.as_ref(),
                    grantee_public_info.account.as_ref(),
                    MSG_SEND_URL,
                )
                .await
            {
                Ok(res) => {
                    let mut found = false;

                    // Get total tx amount in case we find SendAuthorization so that we can verify grant can satisfy tx's
                    let mut tx_amt_total = 0;
                    for tx in toml.transaction.iter() {
                        tx_amt_total += tx.amount;
                    }

                    for grant in res.grants {
                        // Check expiration is valid (either None or at least 1 min of time remaining)
                        if grant.expiration.is_some()
                            && grant.expiration.unwrap().seconds
                                < i64::try_from(
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs()
                                        + 60,
                                )
                                .expect("Could not convert system time to i64")
                        {
                            continue;
                        }

                        // Check grant type is valid now that expiration has been validated
                        if grant.authorization.is_none() {
                            continue;
                        }

                        match grant.authorization.as_ref().unwrap().type_url.as_str() {
                            GENERIC_AUTHORIZATION_URL => {
                                let generic_authorization =
                                    cosmos_modules::authz::GenericAuthorization::decode(
                                        &*grant.authorization.unwrap().value,
                                    )
                                    .expect("Could not decode GenericAuthorization.");

                                if generic_authorization.msg.as_str() != MSG_SEND_URL {
                                    continue;
                                }
                            }
                            SEND_AUTHORIZATION_URL => {
                                // TODO check limit against tx sum
                                let send_authorization =
                                    cosmos_modules::bank::SendAuthorization::decode(
                                        &*grant.authorization.unwrap().value,
                                    )
                                    .expect("Could not decode SendAuthorization.");

                                // Calculate total spend limit
                                // Note: this doesn't mean tx is definitively valid, its possible the spend limit is partially fulfilled already, this is just a rough sanity check.
                                let mut total_spend_limit = 0;

                                for coin in send_authorization.spend_limit {
                                    if coin.denom.as_str() == toml.sender.denom {
                                        total_spend_limit += coin.amount.parse::<u64>().unwrap();
                                    }
                                }

                                if total_spend_limit < tx_amt_total {
                                    continue;
                                }
                            }
                            _ => continue,
                        }

                        // If valid grant found exit
                        found = true;
                        break;
                    }

                    // If no valid grants found, return, otherwise it is implied a valid grant was found.
                    if !found {
                        return Err(AutomatedTxHandlerError::Authorization(String::from(
                            MSG_SEND_URL,
                        )));
                    }
                }
                Err(err) => return Err(AutomatedTxHandlerError::ChainClient(err.to_string())),
            }
        }

        // Build messages to delegate
        let mut msgs: Vec<prost_types::Any> = Vec::new();

        for tx in toml.transaction.iter() {
            let recipient_account_id = match AccountId::from_str(tx.destination_account) {
                Ok(res) => res,
                Err(err) => return Err(AutomatedTxHandlerError::KeyHandling(err.to_string())),
            };

            msgs.push(
                MsgSend {
                    from_address: granter_account_id.clone(),
                    to_address: recipient_account_id,
                    amount: vec![
                        Coin {
                            denom: toml.sender.denom.parse().expect("Could not parse denom."),
                            amount: tx.amount.into(),
                        };
                        1
                    ],
                }
                .to_any()
                .expect("Could not serialize msg."),
            );
        }

        let grantee_base_account = match self
            .query_account(grantee_public_info.account.clone().to_string())
            .await
        {
            Ok(res) => res,
            Err(err) => return Err(AutomatedTxHandlerError::ChainClient(err.to_string())),
        };

        // Send Msg Exec from grantee
        let response = match self
            .execute_authorized_tx(
                Account {
                    id: grantee_public_info.account.clone(),
                    public_key: grantee_public_info.public_key,
                    private_key: self
                        .keyring
                        .get_key(grantee_key_name)
                        .expect("Could not load grantee key."),
                },
                msgs,
                TxMetadata {
                    chain_id: self
                        .config
                        .chain_id
                        .parse()
                        .expect("Could not parse chain id"),
                    account_number: grantee_base_account.account_number,
                    sequence_number: grantee_base_account.sequence,
                    gas_fee: Coin {
                        denom: toml.sender.denom.parse().expect("Could not parse denom."),
                        amount: toml.sender.gas_fee.into(),
                    },
                    gas_limit: toml.sender.gas_limit,
                    timeout_height: toml.sender.timeout_height,
                    memo: toml.sender.memo.to_string(),
                },
                None,
                None,
            )
            .await
        {
            Ok(res) => res,
            Err(err) => return Err(AutomatedTxHandlerError::TxBroadcast(err.to_string())),
        };

        dbg!(response.clone());

        Ok(response)
    }

    pub async fn execute_batch_transactions(
        &mut self,
        toml_path: String,
    ) -> Result<Vec<Response>, AutomatedTxHandlerError> {
        let content = match fs::read_to_string(toml_path) {
            Ok(result) => result,
            Err(err) => {
                return Err(AutomatedTxHandlerError::FileIO(err.to_string()));
            }
        };

        let toml: BatchToml = match toml::from_str(&content) {
            Ok(result) => result,
            Err(err) => {
                return Err(AutomatedTxHandlerError::Toml(err.to_string()));
            }
        };

        dbg!(&toml);

        // Add source key to keyring & parse out relevant types
        let source_key_name = &(String::from("batch") + &Uuid::new_v4().to_string());

        match self.keyring.add_key_from_path(
            source_key_name,
            toml.sender.source_private_key_path,
            false,
        ) {
            Ok(_res) => _res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyStore(err.to_string())),
        };

        let public_info = match self
            .keyring
            .get_public_key_and_address(source_key_name, &self.config.account_prefix)
        {
            Ok(res) => res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyStore(err.to_string())),
        };

        let mut response_vec: Vec<Response> = Vec::new();

        for tx in toml.transactions.iter() {
            let recipient_account_id = match AccountId::from_str(tx.destination_account) {
                Ok(res) => res,
                Err(err) => return Err(AutomatedTxHandlerError::KeyHandling(err.to_string())),
            };

            let tx_base_account = match self
                .query_account(public_info.account.clone().to_string())
                .await
            {
                Ok(res) => res,
                Err(err) => return Err(AutomatedTxHandlerError::ChainClient(err.to_string())),
            };

            let response = match self
                .send(
                    Account {
                        id: public_info.account.clone(),
                        public_key: public_info.public_key,
                        private_key: self
                            .keyring
                            .get_key(source_key_name)
                            .expect("Could not load signing key."),
                    },
                    recipient_account_id,
                    Coin {
                        denom: tx.denom.parse().expect("Could not parse denom."),
                        amount: tx.amount.into(),
                    },
                    TxMetadata {
                        chain_id: self
                            .config
                            .chain_id
                            .parse()
                            .expect("Could not parse chain id"),
                        account_number: tx_base_account.account_number,
                        sequence_number: tx_base_account.sequence,
                        gas_fee: Coin {
                            amount: tx.gas_fee.into(),
                            denom: tx.denom.parse().expect("Could not parse denom."),
                        },
                        gas_limit: tx.gas_limit,
                        timeout_height: tx.timeout_height,
                        memo: tx.memo.to_string(),
                    },
                )
                .await
            {
                Ok(res) => res,
                Err(err) => return Err(AutomatedTxHandlerError::TxBroadcast(err.to_string())),
            };

            response_vec.push(response);
        }

        Ok(response_vec)
    }
}

// ---------------------------------- Tests ----------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use assay::assay;

    #[assay]
    #[test]
    async fn execute_delegated_transacton_toml() {
        // Build and save new toml in test_dir
        let test_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/test_dir");

        // Assert test_dir doesnt exist
        let result = std::panic::catch_unwind(|| std::fs::metadata(test_dir).unwrap());
        assert!(result.is_err());

        // Create test_dir
        let path = Path::new(&test_dir);
        assert!(std::fs::create_dir(path).is_ok());

        // Assert new dir exists now
        assert_eq!(std::fs::metadata(test_dir).unwrap().is_dir(), true);

        let keyring = Keyring::new_file_store(Some(test_dir)).expect("Could not create keyring.");

        let mut chain_client = ChainClient {
            config: ChainClientConfig {
                chain_id: String::from("sommelier-3"),
                rpc_address: String::from("http://localhost:8080"),
                grpc_address: String::from("http://localhost:8080"),
                account_prefix: String::from("somm"),
                gas_adjustment: 1.2,
                gas_prices: String::from("100"),
            },
            keyring: keyring,
            rpc_client: rpc::HttpClient::new("http://localhost:8080")
                .expect("Could not create RPC"),
            cache: None,
        };

        // Assert error if no toml exists
        assert!(chain_client
            .execute_delegated_transacton_toml(String::from("toml_path"), false)
            .await
            .is_err());

        // Ready to create toml assets
        let mut file = DelegatedToml::default();

        // Create source key
        chain_client
            .keyring
            .create_key("Zeus", "", None, true)
            .expect("Could not create signing key.");

        chain_client
            .keyring
            .create_cosmos_key("test_granter_key", "", true)
            .expect("Could not create key.");

        let source_key_path = test_dir.clone() + &String::from("/Zeus.pem");
        file.sender.grantee_private_key_path = source_key_path.as_str();

        let granter_key = chain_client
            .keyring
            .get_public_key_and_address("test_granter_key", "somm")
            .expect("Could not find key.")
            .account;
        file.sender.granter_account = granter_key.as_ref();

        file.sender.denom = "usomm";

        file.sender.gas_fee = 50_000;
        file.sender.gas_limit = 100_000;
        file.sender.timeout_height = 9001u32;
        file.sender.memo = "Delegation memo";

        // Make some transactions
        chain_client
            .keyring
            .create_key("Dionysus", "", None, true)
            .expect("Could not create signing key.");
        let pub_key_output = chain_client
            .keyring
            .get_public_key_and_address("Dionysus", "somm")
            .expect("Could not get public key.");

        file.transaction.push(DelegateTransaction {
            name: "Dionysus",
            destination_account: pub_key_output.account.as_ref(),
            amount: 50u64,
        });

        chain_client
            .keyring
            .create_key("Silenus", "", None, true)
            .expect("Could not create signing key.");
        let pub_key_output = chain_client
            .keyring
            .get_public_key_and_address("Silenus", "somm")
            .expect("Could not get public key.");

        file.transaction.push(DelegateTransaction {
            name: "Silenus",
            destination_account: pub_key_output.account.as_ref(),
            amount: 500u64,
        });

        let toml_string = toml::to_string(&file).expect("Could not encode toml value.");
        let toml_save_path = test_dir.clone() + &String::from("/test_file.toml");

        dbg!(&toml_string);
        dbg!(&toml_save_path);

        fs::write(&toml_save_path, toml_string).expect("Could not write to file.");

        // Execute on toml; expect tx error, but ONLY tx error, everything else should work fine. Tx fails b/c this is unit test so no network connectivity
        let err = chain_client
            .execute_delegated_transacton_toml(toml_save_path, false)
            .await
            .err()
            .unwrap()
            .to_string();

        // Expect Tx error b/c unit test has no network connectivity; do string matching b/c exact error type matching is messy
        assert_eq!(&err[..35], "chain client error: transport error");

        // Clean up dir + toml
        std::fs::remove_dir_all(test_dir)
            .expect(&format!("Failed to delete test directory {}", test_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(test_dir).unwrap());
        assert!(result.is_err());
    }

    #[assay]
    #[test]
    async fn execute_batch_transacton_toml() {
        // Build and save new toml in test_dir
        let test_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/batch_test_dir");

        // Assert test_dir doesnt exist
        let result = std::panic::catch_unwind(|| std::fs::metadata(test_dir).unwrap());
        assert!(result.is_err());

        // Create test_dir
        let path = Path::new(&test_dir);
        assert!(std::fs::create_dir(path).is_ok());

        // Assert new dir exists now
        assert_eq!(std::fs::metadata(test_dir).unwrap().is_dir(), true);

        let keyring = Keyring::new_file_store(Some(test_dir)).expect("Could not create keyring.");

        let mut chain_client = ChainClient {
            config: ChainClientConfig {
                chain_id: String::from("sommelier-3"),
                rpc_address: String::from("http://localhost:8080"),
                grpc_address: String::from("http://localhost:8080"),
                account_prefix: String::from("somm"),
                gas_adjustment: 1.2,
                gas_prices: String::from("100"),
            },
            keyring: keyring,
            rpc_client: rpc::HttpClient::new("http://localhost:8080")
                .expect("Could not create RPC"),
            cache: None,
        };

        // Assert error if no toml exists
        assert!(chain_client
            .execute_batch_transactions(String::from("toml_path"))
            .await
            .is_err());

        // Ready to create toml assets
        let mut file = BatchToml::default();

        // Create source key
        chain_client
            .keyring
            .create_key("Zeus", "", None, true)
            .expect("Could not create signing key.");

        let source_key_path = test_dir.clone() + &String::from("/Zeus.pem");

        file.sender.source_private_key_path = source_key_path.as_str();

        // Make some transactions
        chain_client
            .keyring
            .create_key("Dionysus", "", None, true)
            .expect("Could not create signing key.");
        let pub_key_output = chain_client
            .keyring
            .get_public_key_and_address("Dionysus", "somm")
            .expect("Could not get public key.");

        file.transactions.push(BatchTransaction {
            name: "Dionysus",
            destination_account: pub_key_output.account.as_ref(),
            amount: 50u64,
            denom: "usomm",
            gas_fee: 100_000,
            gas_limit: 100_000,
            timeout_height: 9001u32,
            memo: "Don't spend it all in one place.",
        });

        chain_client
            .keyring
            .create_key("Silenus", "", None, true)
            .expect("Could not create signing key.");
        let pub_key_output = chain_client
            .keyring
            .get_public_key_and_address("Silenus", "somm")
            .expect("Could not get public key.");

        file.transactions.push(BatchTransaction {
            name: "Silenus",
            destination_account: pub_key_output.account.as_ref(),
            amount: 500u64,
            denom: "usomm",
            gas_fee: 100_000,
            gas_limit: 100_000,
            timeout_height: 9001u32,
            memo: "Lorem Ipsum",
        });

        let toml_string = toml::to_string(&file).expect("Could not encode toml value.");
        let toml_save_path = test_dir.clone() + &String::from("/batch_test_file.toml");

        dbg!(&toml_string);
        dbg!(&toml_save_path);

        fs::write(&toml_save_path, toml_string).expect("Could not write to file.");

        // Execute on toml; expect tx error, but ONLY tx error, everything else should work fine. Tx fails b/c this is unit test so no network connectivity
        let err = chain_client
            .execute_batch_transactions(toml_save_path)
            .await
            .err()
            .unwrap()
            .to_string();

        // Expect Tx error b/c unit test has no network connectivity; do string matching b/c exact error type matching is messy
        assert_eq!(&err[..35], "chain client error: transport error");

        // Clean up dir + toml
        std::fs::remove_dir_all(test_dir)
            .expect(&format!("Failed to delete test directory {}", test_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(test_dir).unwrap());
        assert!(result.is_err());
    }
}
