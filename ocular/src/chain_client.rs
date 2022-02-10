use crate::{error::ChainClientError, keys::{KeyOutput, self}};
use bip39::{Mnemonic, Seed};
use serde::{Deserialize, Serialize};
use signatory::FsKeyStore;
use std::path::Path;

#[derive(Default)]
pub struct ChainClient {
    config: ChainClientConfig,
    keybase: Option<FsKeyStore>,
    // KeyringOptions: Vec<?>
    // rpc_client: ?
    // light_provider: ?
    // input:
    // output:
    // codec: ? // needed?
    // logger needed? i think rust does logging differently
}

impl ChainClient {
    pub fn create_key_store(&mut self, path: &str) -> Result<(), ChainClientError> {
        let path = Path::new(path);
        match FsKeyStore::create(path) {
            Ok(ks) =>  {
                self.keybase = Some(ks);
                self.config.key_directory = path.to_str().unwrap().to_string();
            },
            Err(err) => return Err(err.into()),
        };

        Ok(())
    }

    // key_directory and keybase could potentially get into a mistmatched state,
    // this should be checked during create_key_store probably?
    pub fn key_store_created(&self) -> bool {
        self.config.key_directory.is_empty() || self.keybase.is_none()
    }

    pub fn add_key(&self, name: &str) -> String {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let seed = Seed::new(&mnemonic, "");
        format!("{:x}", seed)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChainClientConfig {
    pub key: String,
    pub chain_id: String,
    #[serde(rename = "rpc-addr")]
    pub rpc_address: String,
    #[serde(rename = "grpc-addr")]
    pub grpc_address: String,
    pub account_prefix: String,
    pub keyring_backend: String,
    pub gas_adjustment: f64,
    pub gas_prices: String,
    pub key_directory: String,
}
