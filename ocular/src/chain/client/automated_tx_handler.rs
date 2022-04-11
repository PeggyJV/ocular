#![warn(unused_qualifications)]
#![allow(unused_imports)]
// Clippy broken; doesn't recognize certain imports are used and sees them as unused

use crate::{
    chain::{
        client::tx::{Account, TxMetadata},
        config::ChainClientConfig,
    },
    error::AutomatedTxHandlerError,
    keyring::Keyring,
};
use bip32::Mnemonic;
use cosmrs::{bank::MsgSend, rpc, tx::Msg, AccountId, Coin};

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{fs, path::Path, str::FromStr};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
use uuid::Uuid;

use super::ChainClient;

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
    // TODO: replace account and sequence numbers with pulled numbers from Account type once implemented in https://github.com/PeggyJV/ocular/issues/25
    pub grantee_account_number: u64,
    pub grantee_sequence_number: u64,
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

impl ChainClient {
    // Creates a brand new temporary key for delegated tx workflows.
    pub async fn execute_delegated_transacton_toml(
        &mut self,
        toml_path: String,
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
                    account_number: toml.sender.grantee_account_number,
                    sequence_number: toml.sender.grantee_sequence_number,
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
        };

        // Assert error if no toml exists
        assert!(chain_client
            .execute_delegated_transacton_toml(String::from("toml_path"))
            .await
            .is_err());

        // Ready to create toml assets
        let mut file = DelegatedToml::default();

        // Create source key
        chain_client
            .keyring
            .create_key("Zeus", "", None, true)
            .expect("Could not create signing key.");

        let source_key_path = test_dir.clone() + &String::from("/Zeus.pem");
        file.sender.source_private_key_path = source_key_path.as_str();
        file.sender.delegate_expiration_unix_seconds = i64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 50000,
        )
        .expect("Could not convert to i64");

        file.sender.fee_grant_expiration_unix_seconds = i64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 50000,
        )
        .expect("Could not convert to i64");
        file.sender.fee_grant_amount = 500_000;
        file.sender.denom = "usomm";

        file.sender.grant_account_number = 1;
        file.sender.grant_sequence_number = 0;
        file.sender.grant_gas_fee = 50_000;
        file.sender.grant_gas_limit = 100_000;
        file.sender.grant_timeout_height = 9001u32;
        file.sender.grant_memo = "Delegation memo";

        file.sender.exec_timeout_height = 9001u32;
        file.sender.exec_memo = "Delegation memo";

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
            .execute_delegated_transacton_toml(toml_save_path)
            .await
            .err()
            .unwrap()
            .to_string();

        // Expect Tx error b/c unit test has no network connectivity; do string matching b/c exact error type matching is messy
        assert_eq!(&err[..45], "error sending tx: error broadcasting message:");

        // Clean up dir + toml
        std::fs::remove_dir_all(test_dir)
            .expect(&format!("Failed to delete test directory {}", test_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(test_dir).unwrap());
        assert!(result.is_err());
    }
}
