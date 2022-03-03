#![warn(unused_qualifications)]

use bip32::{Mnemonic, PrivateKey};
use cosmrs::crypto::{secp256k1::SigningKey, PublicKey};
use rand_core::OsRng;
use signatory::{
    pkcs8::der::Document, pkcs8::EncodePrivateKey, pkcs8::LineEnding, FsKeyStore, KeyName,
};
use std::{fs, path::Path};

use crate::error::KeyStoreError;

// Constants
// TODO: Move to independant constants file if reused elsewhere
const COSMOS_BASE_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
const COSMOS_ADDRESS_PREFIX: &str = "cosmos";
const DEFAULT_FS_KEYSTORE_DIR: &str = "/.ocular/keys";

// TODO: Additional hashmap of key attributes: name, prefix, type..
// ADD COSMOS KEYRING FUNCTIONS
// Move keyring functions out of filestore

/// Basic keystore traits that all backends are expected to implement
pub trait KeyStore {
    /// Create new key store
    fn create_key_store(&mut self) -> Result<(), KeyStoreError>;

    /// Check if key store has been initialized.
    fn key_store_created(&self) -> bool;

    /// Check if key exists under specific name. Will return false if no key is found.
    fn key_exists(&self, name: &str) -> Result<bool, KeyStoreError>;

    /// Add a new key based off of name, password, and derivation path (defaults to cosmos). If override_if_exists is set to true, it will override any existing key with the same name.
    fn add_key(
        &self,
        name: &str,
        password: &str,
        derivation_path: Option<&str>,
        override_if_exists: bool,
    ) -> Result<PrivateKeyOutput, KeyStoreError>;

    /// Delete key with a given name. If no key exists under name specified an error will be thrown.
    fn delete_key(&self, name: &str) -> Result<(), KeyStoreError>;

    /// Rename key. If override_if_exists is true any keys with the new key name will be forecfully overriden.
    fn rename_key(
        &self,
        current_name: &str,
        new_name: &str,
        override_if_exists: bool,
    ) -> Result<(), KeyStoreError>;

    /// Get key address in bech32 (aka segwit) format. Will throw an error if the key does not exist.
    fn get_public_key_and_address(&self, name: &str) -> Result<PublicKeyOutput, KeyStoreError>;

    /// Get all key addresses in bech32 (aka segwit) format.
    fn get_all_keys(&self) -> Result<Vec<PublicKeyOutput>, Box<dyn std::error::Error>>;

    /// Recover key via mnemonic, password, and derivation_path (defaults to cosmos). If override_if_exists is set to true, it will override any existing key with the same name.
    fn recover_from_mnemonic(
        &self,
        name: &str,
        mnemonic: &str,
        password: &str,
        derivation_path: Option<&str>,
        override_if_exists: bool,
    ) -> Result<(), KeyStoreError>;
}

/// Mnemonic and private key
pub struct PrivateKeyOutput {
    pub mnemonic: Mnemonic,
    pub private_key: SigningKey,
}

/// Key name and address in Bech32 (aka segwit) format
#[derive(Debug)]
pub struct PublicKeyOutput {
    pub name: String,
    pub public_key: PublicKey,
    pub account: cosmrs::AccountId,
}

/// Keyring that needs to be initialized before being used. Initialization parameters vary depending on type of key store being used.
pub struct Keyring {
    pub key_store: Box<dyn KeyStore>,
}

// Keyring constructors
impl Keyring {
    /// Create new instance of FsKeyStore
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
        })
    }

    // Alternative key store types to be implemented via separate constructors
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

    fn key_exists(&self, name: &str) -> Result<bool, KeyStoreError> {
        let key_name = &KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));

        if !self.key_store_created() {
            return Err(KeyStoreError::NotInitialized);
        }

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
        name: &str,
        password: &str,
        derivation_path: Option<&str>,
        override_if_exists: bool,
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

        // Process key and store
        let extended_signing_key =
            bip32::XPrv::derive_from_path(seed, &derivation_path).expect("Could not derive key.");

        let signing_key = k256::SecretKey::from(extended_signing_key.private_key());
        let encoded_key = signing_key
            .to_pkcs8_der()
            .expect("Could not PKCS8 encode private key");

        let key_name = &KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));
        self.key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(key_name, &encoded_key)
            .expect("Could not store key");

        Ok(PrivateKeyOutput {
            mnemonic,
            private_key: SigningKey::from(extended_signing_key),
        })
    }

    fn delete_key(&self, name: &str) -> Result<(), KeyStoreError> {
        if self.key_exists(name)? {
            let key_name = &KeyName::new(name)
                .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));
            let _delete_key = self
                .key_store
                .as_ref()
                .expect("Error accessing key store.")
                .delete(key_name);

            Ok(())
        } else {
            Err(KeyStoreError::DoesNotExist(name.to_string()))
        }
    }

    fn rename_key(
        &self,
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

        let key = self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .load(current_name)
            .expect("Could not load key.");

        // Create new key.
        let _result = self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(new_name, &key);

        // Delete old key.
        let _result = self.delete_key(current_name);

        Ok(())
    }

    fn get_public_key_and_address(&self, name: &str) -> Result<PublicKeyOutput, KeyStoreError> {
        // Check if key exists
        if !self.key_exists(name)? {
            eprintln!("Key '{}', does not exist.", name);
            return Err(KeyStoreError::DoesNotExist(name.to_string()));
        }

        // Prepare signing key and parse into pem
        let key_name = &KeyName::new(name)
            .unwrap_or_else(|_| panic!("Could not create KeyName for '{}'.", name));
        let signing_key = self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .load(key_name)
            .expect("Could not load key");

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
            .account_id(COSMOS_ADDRESS_PREFIX)
            .expect("Could not get account id from verifying key.");

        Ok(PublicKeyOutput {
            name: name.to_string(),
            public_key: verifying_key,
            account: account_id,
        })
    }

    fn get_all_keys(&self) -> Result<Vec<PublicKeyOutput>, Box<dyn std::error::Error>> {
        let mut vec = Vec::new();

        for entry in fs::read_dir(&self.key_path).expect("Could not read directory.") {
            let path = entry.unwrap().path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "pem" {
                        let name = path
                            .file_stem()
                            .expect("Could not get file stem.")
                            .to_str()
                            .expect("Could not convert to string.");
                        let key_data = self.get_public_key_and_address(name)?;

                        vec.push(key_data);
                    }
                }
            }
        }

        Ok(vec)
    }

    fn recover_from_mnemonic(
        &self,
        name: &str,
        mnemonic: &str,
        password: &str,
        derivation_path: Option<&str>,
        override_if_exists: bool,
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
        self.key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(key_name, &encoded_key)
            .expect("Could not store key");

        Ok(())
    }
}

// ---------------------------------- Tests ----------------------------------
// TODO: Make these tests more comprehensive and increase code coverage.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_key_store_without_path_init() {
        let keyring = Keyring::new_file_store(None).expect("Could not initialize keystore.");

        assert_eq!(keyring.key_store.key_store_created(), true);

        // Assert dir exists where expected
        let expected_dir = String::from(
            dirs::home_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                + DEFAULT_FS_KEYSTORE_DIR,
        );
        assert_eq!(fs::metadata(expected_dir).unwrap().is_dir(), true);

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
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
        assert!(result.is_err());

        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");
        assert_eq!(keyring.key_store.key_store_created(), true);

        // Assert new dir exists now
        assert_eq!(fs::metadata(new_dir).unwrap().is_dir(), true);

        // Clean up dir
        fs::remove_dir(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
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
        assert_eq!(fs::metadata(existing_dir).unwrap().is_dir(), true);

        let keyring =
            Keyring::new_file_store(Some(existing_dir)).expect("Could not initialize keystore.");
        assert_eq!(keyring.key_store.key_store_created(), true);

        // Assert dir still exists
        assert_eq!(fs::metadata(existing_dir).unwrap().is_dir(), true);
    }

    #[test]
    fn file_key_store_add_key() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir1");
        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Check add key doesn't result in failure
        assert!(keyring.key_store.add_key("NewKey", "", None, false).is_ok());

        // Assert attempting to override key results in failure
        assert!(keyring
            .key_store
            .add_key("NewKey", "", None, false)
            .is_err());

        // Assert attempting to override key with override results in success
        assert!(keyring.key_store.add_key("NewKey", "", None, true).is_ok());

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
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
        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Check key doesnt exist
        assert_eq!(keyring.key_store.key_exists("dolphin").unwrap(), false);

        // Create key
        let _new_key = keyring.key_store.add_key("dolphin", "", None, false);

        // Assert new key exists
        assert_eq!(keyring.key_store.key_exists("dolphin").unwrap(), true);

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
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
        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Attempt to delete key that doesnt exist, assert Err thrown
        assert!(keyring.key_store.delete_key("harambe").is_err());

        // Create new key
        let _new_key = keyring.key_store.add_key("harambe", "", None, false);

        // Delete existing key
        assert!(keyring.key_store.delete_key("harambe").is_ok());

        // Verify it was deleted
        assert_eq!(keyring.key_store.key_exists("harambe").unwrap(), false);

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
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
        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Attempt to rename key that doesn't exist
        assert!(keyring
            .key_store
            .rename_key("current_name", "new_name", false)
            .is_err());
        assert!(keyring
            .key_store
            .rename_key("current_name", "new_name", true)
            .is_err());

        // Create some new keys
        let _key = keyring.key_store.add_key("penguin", "", None, false);
        let _key = keyring.key_store.add_key("mouse", "", None, false);

        // Verify keys exists and new named key does not
        assert_eq!(keyring.key_store.key_exists("penguin").unwrap(), true);
        assert_eq!(keyring.key_store.key_exists("mouse").unwrap(), true);
        assert_eq!(keyring.key_store.key_exists("capybara").unwrap(), false);

        // Attempt valid rename without override
        assert!(keyring
            .key_store
            .rename_key("mouse", "capybara", false)
            .is_ok());

        // Verify rename worked
        assert_eq!(keyring.key_store.key_exists("mouse").unwrap(), false);
        assert_eq!(keyring.key_store.key_exists("capybara").unwrap(), true);

        // Attempt rename again into existing key without override and re validate keystore integrity
        assert!(keyring
            .key_store
            .rename_key("capybara", "penguin", false)
            .is_err());
        assert_eq!(keyring.key_store.key_exists("penguin").unwrap(), true);
        assert_eq!(keyring.key_store.key_exists("capybara").unwrap(), true);

        // Attempt rename with valid override
        assert!(keyring
            .key_store
            .rename_key("capybara", "penguin", true)
            .is_ok());

        // Verify rename worked.
        assert_eq!(keyring.key_store.key_exists("capybara").unwrap(), false);
        assert_eq!(keyring.key_store.key_exists("penguin").unwrap(), true);

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
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
        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Attempt to get key address that doesn't exist
        assert!(keyring
            .key_store
            .get_public_key_and_address("iguana")
            .is_err());

        // Make new key
        let key = keyring.key_store.add_key("iguana", "", None, false);

        dbg!(key.unwrap().mnemonic.phrase());

        // Get key address
        let result = keyring.key_store.get_public_key_and_address("iguana");
        assert!(result.is_ok());

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

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
        let result = keyring.key_store.get_all_keys();
        assert!(result.is_ok());
        assert_eq!(keyring.key_store.get_all_keys().unwrap().len(), 0);

        // Make new keys
        let _key = keyring.key_store.add_key("car", "", None, false);
        let _key = keyring.key_store.add_key("motorcycle", "", None, false);

        // Verify new keys
        let result = keyring.key_store.get_all_keys();
        assert!(result.is_ok());
        assert_eq!(keyring.key_store.get_all_keys().unwrap().len(), 2);

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_recover_from_mnemonic() {
        let new_dir = &(std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir7");
        let keyring =
            Keyring::new_file_store(Some(new_dir)).expect("Could not initialize keystore.");

        // Verify key doesn't exist to start
        assert!(keyring
            .key_store
            .get_public_key_and_address("celery")
            .is_err());

        // Create new key and get address
        let private_key = keyring
            .key_store
            .add_key("celery", "tomato", None, false)
            .unwrap();
        let public_key = keyring
            .key_store
            .get_public_key_and_address("celery")
            .unwrap();

        // Delete it
        assert!(keyring.key_store.delete_key("celery").is_ok());
        assert_eq!(keyring.key_store.key_exists("celery").unwrap(), false);

        // Attempt recovery via mnemonic
        assert!(keyring
            .key_store
            .recover_from_mnemonic(
                "new_celery",
                &private_key.mnemonic.phrase(),
                "tomato",
                None,
                false
            )
            .is_ok());

        // Verify recovered key is equal to deleted one
        let new_public_key = keyring
            .key_store
            .get_public_key_and_address("new_celery")
            .unwrap();
        assert_eq!(new_public_key.account.as_ref(), public_key.account.as_ref());
        assert_eq!(new_public_key.public_key, public_key.public_key);

        // Clean up dir
        fs::remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }
}
