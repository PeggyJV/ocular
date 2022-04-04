#![warn(unused_qualifications)]

use crate::{
    chain::{
        client::tx::{Account, TxMetadata},
        config::ChainClientConfig,
    },
    error::{AutomatedTxHandlerError, TxError},
    keyring::Keyring,
};
use bip32::{Mnemonic, PrivateKey};
use cosmrs::{
    crypto::{secp256k1::SigningKey, PublicKey},
    rpc, AccountId, Coin,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use signatory::{pkcs8::der::Document, pkcs8::LineEnding};
use std::time::{Duration, SystemTime};
use std::{fs, path::Path, str::FromStr};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
use uuid::Uuid;

use super::ChainClient;

// Toml structs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DelegatedToml<'a> {
    pub sender: DelegatedSender<'a>,

    #[serde(borrow)]
    pub transactions: Vec<Transaction<'a>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DelegatedSender<'a> {
    pub source_private_key_path: &'a str,
    pub delegate_until_unix_seconds: u64,
}

// TODO: Auto fetch account_number & sequence_number (& potentially gas limit) from account type once https://github.com/PeggyJV/ocular/issues/25 implemented
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Transaction<'a> {
    pub name: &'a str,
    pub destination_account: &'a str,
    pub amount: u64,
    pub denom: &'a str,
    pub account_number: u64,
    pub sequence_number: u64,
    pub gas_fee: u64,
    pub gas_limit: u64,
    pub timeout_height: u32,
    pub memo: &'a str,
}

// Return type for delegated tx workflow
pub struct DelegatedTransactionOutput {
    pub delegated_mnemonic: Mnemonic,
    pub response: Vec<Response>,
}

impl ChainClient {
    // Creates a brand new key
    pub async fn execute_delegated_transacton_toml(
        &mut self,
        toml_path: String,
    ) -> Result<DelegatedTransactionOutput, AutomatedTxHandlerError> {
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


        let granter_key_name = &Uuid::new_v4().to_string();

        match self.keyring.add_key_from_path(granter_key_name, toml.sender.source_private_key_path, false)
        {
            Ok(_res) => _res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyStore(err.to_string())),
        };


        let granter_public_info = match self.keyring.get_public_key_and_address(granter_key_name, &self.config.account_prefix)
        {
            Ok(res) => res,
            Err(err) => return Err(AutomatedTxHandlerError::KeyStore(err.to_string())),
        };






        
        Ok(DelegatedTransactionOutput {
            delegated_mnemonic: Mnemonic::random(&mut OsRng, Default::default()),
            response: Vec::new(),
        })
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
        file.sender.delegate_until_unix_seconds = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 50000;

        // Make some transactions
        chain_client
            .keyring
            .create_key("Dionysus", "", None, true)
            .expect("Could not create signing key.");
        let pub_key_output = chain_client
            .keyring
            .get_public_key_and_address("Dionysus", "somm")
            .expect("Could not get public key.");

        file.transactions.push(Transaction {
            name: "Dionysus",
            destination_account: pub_key_output.account.as_ref(),
            amount: 50u64,
            denom: "usomm",
            account_number: 1,
            sequence_number: 0,
            gas_fee: 50_000,
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

        file.transactions.push(Transaction {
            name: "Silenus",
            destination_account: pub_key_output.account.as_ref(),
            amount: 500u64,
            denom: "usomm",
            account_number: 1,
            sequence_number: 0,
            gas_fee: 50_000,
            gas_limit: 100_000,
            timeout_height: 9001u32,
            memo: "Lorem Ipsum",
        });

        let toml_string = toml::to_string(&file).expect("Could not encode toml value.");
        let toml_save_path = /*test_dir.clone()*/ String::from("/Users/phil/Desktop/peggyJV/ocular") + &String::from("/test_file.toml");

        dbg!(&toml_string);
        dbg!(&toml_save_path);

        fs::write(&toml_save_path, toml_string).expect("Could not write to file.");

        // Execute on toml; expect tx error, but ONLY tx error, everything else should work fine. Tx fails b/c this is unit test so no network connectivity
        dbg!(
            chain_client
                .execute_delegated_transacton_toml(toml_save_path)
                .await
                .unwrap()
                .response
        );
        /*
        let err = chain_client
            .execute_delegated_transacton_toml(toml_save_path)
            .await
            .err()
            .unwrap()
            .to_string();
            */
    }
}
