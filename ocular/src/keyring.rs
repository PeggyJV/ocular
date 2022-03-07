#![warn(unused_qualifications)]
#![allow(dead_code)]
// TODO: Remove dead code allowance once these methods are used elsewhere

use bip32::{Mnemonic, PrivateKey};
use cosmrs::crypto::{secp256k1::SigningKey, PublicKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use signatory::{
    pkcs8::der::Document, pkcs8::EncodePrivateKey, pkcs8::LineEnding, FsKeyStore, KeyName,
};
use std::collections::HashMap;
use std::path::Path;

use crate::error::KeyStoreError;

// Constants
// TODO: Move to independant constants file if reused elsewhere
const COSMOS_BASE_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
const DEFAULT_FS_KEYSTORE_DIR: &str = "/.ocular/keys";

/// Basic keystore traits that all backends are expected to implement
pub trait KeyStore {
    /// Create new key store
    fn create_key_store(&mut self) -> Result<(), KeyStoreError>;

    /// Check if key store has been initialized.
    fn key_store_created(&self) -> bool;

    /// Check if key exists under specific name. Will return false if no key is found.
    fn key_exists(&self, keyname: &KeyName) -> Result<bool, KeyStoreError>;

    /// Add a private key document associated with a key name into the keystore.
    fn add_key(
        &self,
        key_name: &KeyName,
        encoded_key: pkcs8::PrivateKeyDocument,
    ) -> Result<(), KeyStoreError>;

    /// Delete key with a given name. If no key exists under name specified an error will be thrown.
    fn delete_key(&self, key_name: &KeyName) -> Result<(), KeyStoreError>;

    /// Rename key.
    fn rename_key(&self, current_name: &KeyName, new_name: &KeyName) -> Result<(), KeyStoreError>;

    /// Load PrivateKeyDocumentfrom key store. Will return an error if key DNE under name.
    fn get_key(&self, key_name: &KeyName) -> Result<pkcs8::PrivateKeyDocument, KeyStoreError>;

    /*
        /// Get all key addresses in bech32 (aka segwit) format.
        fn get_all_keys(&self) -> Result<Vec<PublicKeyOutput>, Box<dyn std::error::Error>>;
    */
}

/// Mnemonic and private key
pub struct PrivateKeyOutput {
    pub mnemonic: Mnemonic,
    pub private_key: SigningKey,
}

/// Key name and address in Bech32 (aka segwit) format
#[derive(Debug)]
pub struct PublicKeyOutput {
    /// Name must be unique.
    pub name: String,
    pub public_key: PublicKey,
    pub account: cosmrs::AccountId,
}

// TODO: As needed, support various record type storage and retrieval, currently it is purely superficial
/// Various kidns of key record types.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecordType {
    Local,
    Ledger,
    Multi,
    Offline,
}

/// Key records to be used by the keyring.
#[derive(Clone, Debug)]
pub struct Record {
    pub record_type: RecordType,
}

/// Keyring that needs to be initialized before being used. Initialization parameters vary depending on type of key store being used.
pub struct Keyring {
    key_store: Box<dyn KeyStore>,
    records: HashMap<String, Record>,
}

// Keyring constructors
impl Keyring {
    /// Create new instance of FsKeyStore.
    /// Will create store at '~/<DEFAULT_FS_KEYSTORE_DIR>' if None is provided
    pub fn new_file_store(key_path: Option<&str>) -> Result<Self, KeyStoreError> {
        let path: String;

        if let Some(key_path) = key_path {
            path = key_path.to_string();
        } else {
            path = dirs::home_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                + DEFAULT_FS_KEYSTORE_DIR;
        }

        dbg!(format!("Attempting to use path {}", path));

        let mut key_store = FileKeyStore {
            key_path: path,
            key_store: None,
        };

        key_store.create_key_store()?;

        Ok(Keyring {
            key_store: Box::new(key_store),
            records: HashMap::new(),
        })
    }

    // Alternative key store types to be implemented via separate constructors

    // ----- Keyring utilities -----

    /// Check if key store has been initialized.
    pub fn key_store_created(&self) -> bool {
        self.key_store.key_store_created()
    }

    /// Check if key exists under specific name. Will return false if no key is found.
    pub fn key_exists(&self, name: &str) -> Result<bool, KeyStoreError> {
        let key_name = &KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));

        if !self.key_store_created() {
            return Err(KeyStoreError::NotInitialized);
        }

        self.key_store.key_exists(key_name)
    }

    /// Add a new key based off of name, password, and derivation path (defaults to cosmos); a mnemonic will automatically be created. If override_if_exists is set to true, it will override any existing key with the same name.
    pub fn add_key_with_generated_mnemonic(
        &mut self,
        name: &str,
        password: &str,
        derivation_path: Option<&str>,
        override_if_exists: bool,
        record_type: RecordType,
    ) -> Result<PrivateKeyOutput, KeyStoreError> {
        // Check if key already exists
        if self.key_exists(name)? && !override_if_exists {
            eprintln!("Key '{}', already exists.", name);
            return Err(KeyStoreError::Exists(name.to_string()));
        }

        let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
        let seed = mnemonic.to_seed(password);

        let derivation_path = match derivation_path {
            Some(_i) => derivation_path.unwrap(),
            _ => COSMOS_BASE_DERIVATION_PATH,
        };

        let derivation_path = derivation_path
            .parse::<bip32::DerivationPath>()
            .expect("Could not parse derivation path.");

        // Process key
        let extended_signing_key =
            bip32::XPrv::derive_from_path(seed, &derivation_path).expect("Could not derive key.");

        let signing_key = k256::SecretKey::from(extended_signing_key.private_key());
        let encoded_key = signing_key
            .to_pkcs8_der()
            .expect("Could not PKCS8 encode private key");

        let key_name = KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));

        // Store the key
        self.key_store.add_key(&key_name, encoded_key)?;

        self.records
            .insert(String::from(name), Record { record_type });

        Ok(PrivateKeyOutput {
            mnemonic,
            private_key: SigningKey::from(extended_signing_key),
        })
    }

    /// Delete key with a given name. If no key exists under name specified an error will be thrown.
    fn delete_key(&mut self, name: &str) -> Result<(), KeyStoreError> {
        if self.key_exists(name)? {
            let key_name = &KeyName::new(name)
                .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));

            self.key_store.delete_key(key_name)?;

            self.records.remove(&String::from(name));

            Ok(())
        } else {
            Err(KeyStoreError::DoesNotExist(name.to_string()))
        }
    }

    /// Rename key. If override_if_exists is true any keys with the new key name will be forecfully overriden. Errors will be thrown if current name DNE or if new key name already exists without override flag being set to true.
    pub fn rename_key(
        &mut self,
        current_name: &str,
        new_name: &str,
        override_if_exists: bool,
    ) -> Result<(), KeyStoreError> {
        // Check if current key exists
        if !self.key_exists(current_name)? {
            eprintln!("Key '{}', does not exist.", current_name);
            return Err(KeyStoreError::DoesNotExist(current_name.to_string()));
        }

        // Check if new key exists
        if self.key_exists(new_name)? && !override_if_exists {
            eprintln!("New key name '{}', already exists.", new_name);
            return Err(KeyStoreError::Exists(new_name.to_string()));
        }

        // Proceed with rename
        let current_name = &KeyName::new(current_name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", current_name));
        let new_name = &KeyName::new(new_name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", new_name));

        self.key_store.rename_key(current_name, new_name)?;

        self.records.insert(
            new_name.to_string(),
            self.records
                .get(&current_name.to_string())
                .expect("No record for key name.")
                .clone(),
        );
        self.records.remove(&current_name.to_string());

        Ok(())
    }

    /// Get key address in bech32 (aka segwit) format. Will throw an error if the key does not exist.
    pub fn get_public_key_and_address(
        &self,
        name: &str,
        prefix: &str,
    ) -> Result<PublicKeyOutput, KeyStoreError> {
        // Check if key exists
        if !self.key_exists(name)? {
            eprintln!("Key '{}', does not exist.", name);
            return Err(KeyStoreError::DoesNotExist(name.to_string()));
        }

        // Prepare signing key and parse into pem
        let key_name = &KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));

        let signing_key = self.key_store.get_key(key_name)?;

        let signing_key: k256::elliptic_curve::SecretKey<k256::Secp256k1> = signing_key
            .to_pem(LineEnding::default())
            .expect("Could not convert to pem.")
            .parse()
            .expect("Could not parse pem");

        let signing_key_bytes = signing_key.to_bytes();

        let verifying_key = SigningKey::from_bytes(&signing_key_bytes)
            .expect("Could not create verifying key from signing key.")
            .public_key();

        // TODO: Support other prefixes
        let account_id = verifying_key
            .account_id(prefix)
            .expect("Could not get account id from verifying key.");

        Ok(PublicKeyOutput {
            name: name.to_string(),
            public_key: verifying_key,
            account: account_id,
        })
    }
    /*
        /// Get all key addresses in bech32 (aka segwit) format.
        pub fn get_all_keys(&self) -> Result<Vec<PublicKeyOutput>, Box<dyn std::error::Error>> {
            self.key_store.get_all_keys()
        }
    */

    /// Recover key via mnemonic, password, and derivation_path (defaults to cosmos). If override_if_exists is set to true, it will override any existing key with the same name.
    fn create_or_recover_from_mnemonic(
        &mut self,
        name: &str,
        mnemonic: &str,
        password: &str,
        derivation_path: Option<&str>,
        override_if_exists: bool,
        record_type: RecordType,
    ) -> Result<(), KeyStoreError> {
        // Check if key already exists
        if self.key_exists(name)? && !override_if_exists {
            eprintln!("Key '{}', already exists.", name);
            return Err(KeyStoreError::Exists(name.to_string()));
        }

        let mnemonic = Mnemonic::new(mnemonic.trim(), Default::default())
            .map_err(|err| KeyStoreError::InvalidMnemonic(err.to_string()))?;
        let seed = mnemonic.to_seed(password);

        let derivation_path = match derivation_path {
            Some(_i) => derivation_path.unwrap(),
            _ => COSMOS_BASE_DERIVATION_PATH,
        };

        let derivation_path = derivation_path
            .parse::<bip32::DerivationPath>()
            .expect("Could not parse derivation path.");

        // Process key and store
        let key =
            bip32::XPrv::derive_from_path(seed, &derivation_path).expect("Could not derive key.");
        let private_key = k256::SecretKey::from(key.private_key());
        let encoded_key = private_key
            .to_pkcs8_der()
            .expect("Could not PKCS8 encode private key");

        let key_name = &KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));

        self.key_store.add_key(key_name, encoded_key)?;

        self.records
            .insert(String::from(name), Record { record_type });

        Ok(())
    }
}

// --- File Key Store ---
pub struct FileKeyStore {
    key_path: String,
    key_store: Option<FsKeyStore>,
}

impl KeyStore for FileKeyStore {
    fn create_key_store(&mut self) -> Result<(), KeyStoreError> {
        let path = Path::new(&self.key_path);

        match FsKeyStore::create_or_open(path) {
            Ok(ks) => {
                self.key_store = Some(ks);
            }
            Err(err) => return Err(KeyStoreError::CouldNotOpenOrCreateKeyStore(err.to_string())),
        };

        Ok(())
    }

    fn key_store_created(&self) -> bool {
        !self.key_path.is_empty() && self.key_store.is_some()
    }

    fn key_exists(&self, key_name: &KeyName) -> Result<bool, KeyStoreError> {
        if let Ok(_info) = self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .info(key_name)
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn add_key(
        &self,
        key_name: &KeyName,
        encoded_key: pkcs8::PrivateKeyDocument,
    ) -> Result<(), KeyStoreError> {
        return match self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(key_name, &encoded_key)
        {
            Ok(_ks) => Ok(()),
            Err(err) => Err(KeyStoreError::UnableToStoreKey(err.to_string())),
        };
    }

    fn delete_key(&self, key_name: &KeyName) -> Result<(), KeyStoreError> {
        return match self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .delete(key_name)
        {
            Ok(_ks) => Ok(()),
            Err(err) => Err(KeyStoreError::UnableToDeleteKey(err.to_string())),
        };
    }

    fn rename_key(&self, current_name: &KeyName, new_name: &KeyName) -> Result<(), KeyStoreError> {
        let key = self.get_key(current_name).expect("Could not load key.");

        // Store new key.
        self.key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(new_name, &key)
            .expect("Could not create new key.");

        // Delete old key.
        self.delete_key(current_name)
            .expect("Could not delete old key.");

        Ok(())
    }

    fn get_key(&self, key_name: &KeyName) -> Result<pkcs8::PrivateKeyDocument, KeyStoreError> {
        return match self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .load(key_name)
        {
            Ok(ks) => Ok(ks),
            Err(err) => Err(KeyStoreError::UnableToRetrieveKey(err.to_string())),
        };
    }
    /*
        fn get_all_keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
            let mut vec = Vec::new();

            for entry in std::fs::read_dir(&self.key_path).expect("Could not read directory.") {
                let path = entry.unwrap().path();

                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "pem" {
                            let name = path
                                .file_stem()
                                .expect("Could not get file stem.")
                                .to_str()
                                .expect("Could not convert to string.");
                            let key_data = super::get_public_key_and_address(name)?;

                            vec.push(key_data);
                        }
                    }
                }
            }

            Ok(vec)
        }
    */
}

// ---------------------------------- Tests ----------------------------------
// TODO: Make these tests more comprehensive and increase code coverage.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_key_store_without_path_init() {
        let keyring = Keyring::new_file_store(None).expect("Could not initialize keystore.");

        assert_eq!(keyring.key_store_created(), true);

        // Assert dir exists where expected
        let expected_dir = String::from(
            dirs::home_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                + DEFAULT_FS_KEYSTORE_DIR,
        );
        assert_eq!(std::fs::metadata(expected_dir).unwrap().is_dir(), true);

        // Don't delete dir in case user already has keys loaded and runs this test
    }

    #[test]
    fn file_key_store_with_new_path_init() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir0");

        // Assert doesnt exist
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());

        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");
        assert_eq!(keyring.key_store_created(), true);

        // Assert new dir exists now
        assert_eq!(std::fs::metadata(new_dir).unwrap().is_dir(), true);

        // Clean up dir
        std::fs::remove_dir(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_key_store_with_existing_path_init() {
        let existing_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap());

        // Assert dir exists
        assert_eq!(std::fs::metadata(existing_dir).unwrap().is_dir(), true);

        let keyring =
            Keyring::new_file_store(Some(existing_dir)).expect("Could not initialize keystore.");
        assert_eq!(keyring.key_store_created(), true);

        // Assert dir still exists
        assert_eq!(std::fs::metadata(existing_dir).unwrap().is_dir(), true);
    }

    #[test]
    fn file_key_store_add_key() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir1");
        let mut keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Check add key doesn't result in failure
        assert!(keyring
            .add_key_with_generated_mnemonic("NewKey", "", None, false, RecordType::Offline)
            .is_ok());

        // Assert attempting to override key results in failure
        assert!(keyring
            .add_key_with_generated_mnemonic("NewKey", "", None, false, RecordType::Offline)
            .is_err());

        // Assert attempting to override key with override results in success
        assert!(keyring
            .add_key_with_generated_mnemonic("NewKey", "", None, true, RecordType::Offline)
            .is_ok());

        // Clean up dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_key_exists() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir2");
        let mut keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Check key doesnt exist
        assert_eq!(keyring.key_exists("dolphin").unwrap(), false);

        // Create key
        let _new_key = keyring.add_key_with_generated_mnemonic(
            "dolphin",
            "",
            None,
            false,
            RecordType::Offline,
        );

        // Assert new key exists
        assert_eq!(keyring.key_exists("dolphin").unwrap(), true);

        // Clean up dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_delete_key() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir3");
        let mut keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Attempt to delete key that doesnt exist, assert Err thrown
        assert!(keyring.delete_key("harambe").is_err());

        // Create new key
        let _new_key = keyring.add_key_with_generated_mnemonic(
            "harambe",
            "",
            None,
            false,
            RecordType::Offline,
        );

        // Delete existing key
        assert!(keyring.delete_key("harambe").is_ok());

        // Verify it was deleted
        assert_eq!(keyring.key_exists("harambe").unwrap(), false);

        // Clean up dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_rename_key() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir4");
        let mut keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Attempt to rename key that doesn't exist
        assert!(keyring
            .rename_key("current_name", "new_name", false)
            .is_err());
        assert!(keyring
            .rename_key("current_name", "new_name", true)
            .is_err());

        // Create some new keys
        let _key = keyring.add_key_with_generated_mnemonic(
            "penguin",
            "",
            None,
            false,
            RecordType::Offline,
        );
        let _key =
            keyring.add_key_with_generated_mnemonic("mouse", "", None, false, RecordType::Offline);

        // Verify keys exists and new named key does not
        assert_eq!(keyring.key_exists("penguin").unwrap(), true);
        assert_eq!(keyring.key_exists("mouse").unwrap(), true);
        assert_eq!(keyring.key_exists("capybara").unwrap(), false);

        // Attempt valid rename without override
        assert!(keyring.rename_key("mouse", "capybara", false).is_ok());

        // Verify rename worked
        assert_eq!(keyring.key_exists("mouse").unwrap(), false);
        assert_eq!(keyring.key_exists("capybara").unwrap(), true);

        // Attempt rename again into existing key without override and re validate keystore integrity
        assert!(keyring.rename_key("capybara", "penguin", false).is_err());
        assert_eq!(keyring.key_exists("penguin").unwrap(), true);
        assert_eq!(keyring.key_exists("capybara").unwrap(), true);

        // Attempt rename with valid override
        assert!(keyring.rename_key("capybara", "penguin", true).is_ok());

        // Verify rename worked.
        assert_eq!(keyring.key_exists("capybara").unwrap(), false);
        assert_eq!(keyring.key_exists("penguin").unwrap(), true);

        // Clean up dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_get_public_key_and_address() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir5");
        let mut keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Attempt to get key address that doesn't exist
        assert!(keyring
            .get_public_key_and_address("iguana", "cosmos")
            .is_err());

        // Make new key
        let key =
            keyring.add_key_with_generated_mnemonic("iguana", "", None, false, RecordType::Offline);

        dbg!(key.unwrap().mnemonic.phrase());

        // Get key address
        let result = keyring.get_public_key_and_address("iguana", "cosmos");
        assert!(result.is_ok());

        // Clean up dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }
    /*
        #[test]
        fn file_store_get_all_keys() {
            let new_dir = &(std::env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                + "/working_test_dir6");
            let keyring =
                Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

            // Verify no keys at start
            let result = keyring.get_all_keys();
            assert!(result.is_ok());
            assert_eq!(keyring.get_all_keys().unwrap().len(), 0);

            // Make new keys
            let _key = keyring.add_key_with_generated_mnemonic("car", "", None, false);
            let _key = keyring.add_key_with_generated_mnemonic("motorcycle", "", None, false);

            // Verify new keys
            let result = keyring.get_all_keys();
            assert!(result.is_ok());
            assert_eq!(keyring.get_all_keys().unwrap().len(), 2);

            // Clean up dir
            std::fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

            // Assert deleted
            let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
            assert!(result.is_err());
        }
    */
    #[test]
    fn file_store_create_or_recover_from_mnemonic() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir7");
        let mut keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Verify key doesn't exist to start
        assert!(keyring
            .get_public_key_and_address("celery", "cosmos")
            .is_err());

        // Create new key and get address
        let private_key = keyring
            .add_key_with_generated_mnemonic("celery", "tomato", None, false, RecordType::Offline)
            .unwrap();
        let public_key = keyring
            .get_public_key_and_address("celery", "cosmos")
            .unwrap();

        // Delete it
        assert!(keyring.delete_key("celery").is_ok());
        assert_eq!(keyring.key_exists("celery").unwrap(), false);

        // Attempt recovery via mnemonic
        assert!(keyring
            .create_or_recover_from_mnemonic(
                "new_celery",
                &private_key.mnemonic.phrase(),
                "tomato",
                None,
                false,
                RecordType::Offline,
            )
            .is_ok());

        // Verify recovered key is equal to deleted one
        let new_public_key = keyring
            .get_public_key_and_address("new_celery", "cosmos")
            .unwrap();
        assert_eq!(new_public_key.account.as_ref(), public_key.account.as_ref());
        assert_eq!(new_public_key.public_key, public_key.public_key);

        // Clean up dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }
}
