use bip32::PrivateKey;
use bip39::{Mnemonic, Seed};
use serde::{Deserialize, Serialize};
use signatory::{
    pkcs8::der::Document, pkcs8::EncodePrivateKey, pkcs8::LineEnding, pkcs8::PrivateKeyDocument,
    FsKeyStore, KeyName,
};
use std::{env, fs, fs::metadata, fs::remove_dir, fs::remove_dir_all, path::Path};

use crate::error::KeyStoreError;

// Constants
// TODO: Move to independant constants file if reused elsewhere
static COSMOS_BASE_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
static COSMOS_ADDRESS_PREFIX: &str = "cosmos";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KeyRingType {
    File,
    // TODO: OS (and other) Types
}

impl Default for KeyRingType {
    fn default() -> Self {
        KeyRingType::File
    }
}

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
    fn get_key_address(&self, name: &str) -> Result<PublicKeyOutput, KeyStoreError>;

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

/// Mnemonic and private key in pkcs8 PrivateKeyDocument format
#[derive(Debug)]
pub struct PrivateKeyOutput {
    mnemonic: String,
    private_key: PrivateKeyDocument,
}

/// Key name and address in Bech32 (aka segwit) format
// TODO: Include actual public key here if ever needed
#[derive(Debug)]
pub struct PublicKeyOutput {
    name: String,
    address: String,
}

// --- Base Key Ring ---
pub struct Keyring {
    backend: KeyRingType,
    pub key_store: Box<dyn KeyStore>,
}

// Keyring constructors
impl Keyring {
    /// Create new instance of FsKeyStore Keyring
    /// Will create store at current working dir if None is provided
    fn new_file_store(key_path: Option<&str>) -> Self {
        let path: String;
        if key_path.is_none() {
            path = env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .expect("Could not get current working dir.");
        } else {
            path = key_path.unwrap().to_string();
        }

        dbg!(format!("Attempting to use path {}", path));

        let mut key_store = FileKeyStore {
            key_path: path.clone(),
            key_store: None,
        };

        key_store.create_key_store().expect(&format!(
            "Could not create file key store for path: {}",
            path
        ));

        Keyring {
            backend: KeyRingType::File,
            key_store: Box::new(key_store),
        }
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
            Err(err) => return Err(err.into()),
        };

        Ok(())
    }

    fn key_store_created(&self) -> bool {
        !self.key_path.is_empty() && !self.key_store.is_none()
    }

    fn key_exists(&self, name: &str) -> Result<bool, KeyStoreError> {
        let key_name =
            &KeyName::new(name).expect(&format!("Could not create KeyName for '{}'.", name));

        if !self.key_store_created() {
            return Err(KeyStoreError::KeyStoreNotInitialized());
        }

        if let Ok(_info) = self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .info(key_name)
        {
            return Ok(true);
        } else {
            return Ok(false);
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
            return Err(KeyStoreError::KeyExists(name.to_string()));
        }

        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let seed = Seed::new(&mnemonic, &password);

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

        let key_name =
            &KeyName::new(name).expect(&format!("Could not create KeyName for '{}'.", name));
        self.key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(&key_name, &encoded_key)
            .expect("Could not store key");

        Ok(PrivateKeyOutput {
            mnemonic: mnemonic.phrase().to_string(),
            private_key: encoded_key,
        })
    }

    fn delete_key(&self, name: &str) -> Result<(), KeyStoreError> {
        if self.key_exists(name)? {
            let key_name =
                &KeyName::new(name).expect(&format!("Could not create KeyName for '{}'.", name));
            let _delete_key = self
                .key_store
                .as_ref()
                .expect("Error accessing key store.")
                .delete(&key_name);

            return Ok(());
        } else {
            return Err(KeyStoreError::KeyDoesNotExist(name.to_string()));
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
            return Err(KeyStoreError::KeyDoesNotExist(current_name.to_string()));
        }

        // Check if new key exists
        if self.key_exists(new_name)? && !override_if_exists {
            eprintln!("New key name '{}', already exists.", new_name);
            return Err(KeyStoreError::KeyExists(new_name.to_string()));
        }

        // Proceed with rename
        let current_name = &KeyName::new(current_name)
            .expect(&format!("Could not create KeyName for '{}'.", current_name));
        let new_name = &KeyName::new(new_name)
            .expect(&format!("Could not create KeyName for '{}'.", new_name));

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
            .store(&new_name, &key);

        // Delete old key.
        let _result = self.delete_key(&current_name);

        Ok(())
    }

    fn get_key_address(&self, name: &str) -> Result<PublicKeyOutput, KeyStoreError> {
        // Check if key exists
        if !self.key_exists(name)? {
            eprintln!("Key '{}', does not exist.", name);
            return Err(KeyStoreError::KeyDoesNotExist(name.to_string()));
        }

        // Prepare key and parse into pem
        let key_name =
            &KeyName::new(name).expect(&format!("Could not create KeyName for '{}'.", name));
        let key = self
            .key_store
            .as_ref()
            .expect("Error accessing key store.")
            .load(&key_name)
            .expect("Could not load key");
        let key: k256::elliptic_curve::SecretKey<k256::Secp256k1> = key
            .to_pem(LineEnding::default())
            .expect("Could not convert to pem.")
            .parse()
            .expect("Could not parse pem");

        // Convert to deepspace private key for address conversion
        let key_bytes = key.to_bytes();
        let key = deep_space::utils::bytes_to_hex_str(&key_bytes);
        let deep_space_key: deep_space::private_key::PrivateKey =
            key.parse().expect("Could not parse private key.");

        // Finally get address
        // TODO: Support other prefixes if necessary
        let address = deep_space_key
            .to_address(COSMOS_ADDRESS_PREFIX)
            .expect("Could not generate address.")
            .to_bech32(COSMOS_ADDRESS_PREFIX)
            .expect("Could not bech32 encode address.");

        return Ok(PublicKeyOutput {
            name: name.to_string(),
            address: address,
        });
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
                        let key_data = self.get_key_address(name)?;

                        vec.push(key_data);
                    }
                }
            }
        }

        return Ok(vec);
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
            return Err(KeyStoreError::KeyExists(name.to_string()));
        }

        let seed = Seed::new(
            &bip39::Mnemonic::from_phrase(mnemonic.trim(), bip39::Language::English)
                .expect("Could not read mnemonic."),
            &password,
        );

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

        let key_name =
            &KeyName::new(name).expect(&format!("Could not create KeyName for '{}'.", name));
        self.key_store
            .as_ref()
            .expect("Error accessing key store.")
            .store(&key_name, &encoded_key)
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
    fn file_key_store_without_path_init_test() {
        let keyring = Keyring::new_file_store(Option::None);

        assert_eq!(keyring.key_store.key_store_created(), true);
    }

    #[test]
    fn file_key_store_with_new_path_init_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir0");

        // Assert doesnt exist
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());

        let keyring = Keyring::new_file_store(Option::Some(new_dir));
        assert_eq!(keyring.key_store.key_store_created(), true);

        // Assert new dir exists now
        assert_eq!(metadata(new_dir).unwrap().is_dir(), true);

        // Clean up dir
        remove_dir(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_key_store_with_existing_path_init_test() {
        let existing_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap());

        // Assert dir exists
        assert_eq!(metadata(existing_dir).unwrap().is_dir(), true);

        let keyring = Keyring::new_file_store(Option::Some(existing_dir));
        assert_eq!(keyring.key_store.key_store_created(), true);

        // Assert dir still exists
        assert_eq!(metadata(existing_dir).unwrap().is_dir(), true);
    }

    #[test]
    fn file_key_store_add_key_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir1");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

        // Check add key doesn't result in failure
        assert!(keyring
            .key_store
            .add_key("NewKey", "", Option::None, false)
            .is_ok());

        // Assert attempting to override key results in failure
        assert!(keyring
            .key_store
            .add_key("NewKey", "", Option::None, false)
            .is_err());

        // Assert attempting to override key with override results in success
        assert!(keyring
            .key_store
            .add_key("NewKey", "", Option::None, true)
            .is_ok());

        // Clean up dir
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_key_exists_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir2");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

        // Check key doesnt exist
        assert_eq!(keyring.key_store.key_exists("dolphin").unwrap(), false);

        // Create key
        let _new_key = keyring
            .key_store
            .add_key("dolphin", "", Option::None, false);

        // Assert new key exists
        assert_eq!(keyring.key_store.key_exists("dolphin").unwrap(), true);

        // Clean up dir
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_delete_key_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir3");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

        // Attempt to delete key that doesnt exist, assert Err thrown
        assert!(keyring.key_store.delete_key("harambe").is_err());

        // Create new key
        let _new_key = keyring
            .key_store
            .add_key("harambe", "", Option::None, false);

        // Delete existing key
        assert!(keyring.key_store.delete_key("harambe").is_ok());

        // Verify it was deleted
        assert_eq!(keyring.key_store.key_exists("harambe").unwrap(), false);

        // Clean up dir
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_rename_key_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir4");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

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
        let key = keyring
            .key_store
            .add_key("penguin", "", Option::None, false);
        let key = keyring.key_store.add_key("mouse", "", Option::None, false);

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
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_get_key_address_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir5");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

        // Attempt to get key address that doesn't exist
        assert!(keyring.key_store.get_key_address("iguana").is_err());

        // Make new key
        let key = keyring.key_store.add_key("iguana", "", Option::None, false);

        // Get key address
        let result = keyring.key_store.get_key_address("iguana");
        assert!(result.is_ok());
        dbg!(result);

        // Clean up dir
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_get_all_keys_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir6");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

        // Verify no keys at start
        let result = keyring.key_store.get_all_keys();
        assert!(result.is_ok());
        assert_eq!(keyring.key_store.get_all_keys().unwrap().len(), 0);

        // Make new keys
        let _key = keyring.key_store.add_key("car", "", Option::None, false);
        let _key = keyring
            .key_store
            .add_key("motorcycle", "", Option::None, false);

        // Verify new keys
        let result = keyring.key_store.get_all_keys();
        assert!(result.is_ok());
        assert_eq!(keyring.key_store.get_all_keys().unwrap().len(), 2);

        dbg!(result);

        // Clean up dir
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_store_recover_from_mnemonic_test() {
        let new_dir = &(env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/working_test_dir7");
        let keyring = Keyring::new_file_store(Option::Some(new_dir));

        // Verify key doesn't exist to start
        assert!(keyring.key_store.get_key_address("celery").is_err());

        // Create new key and get address
        let private_key = keyring
            .key_store
            .add_key("celery", "tomato", Option::None, false)
            .unwrap();
        let public_key = keyring.key_store.get_key_address("celery").unwrap();

        // Delete it
        assert!(keyring.key_store.delete_key("celery").is_ok());
        assert_eq!(keyring.key_store.key_exists("celery").unwrap(), false);

        // Attempt recovery via mneumonic
        assert!(keyring
            .key_store
            .recover_from_mnemonic(
                "new_celery",
                &private_key.mnemonic.to_string(),
                "tomato",
                Option::None,
                false
            )
            .is_ok());

        // Verify recovered key is equal to deleted one
        let new_public_key = keyring.key_store.get_key_address("new_celery").unwrap();
        assert_eq!(new_public_key.address, public_key.address);

        // Clean up dir
        remove_dir_all(new_dir).expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| metadata(new_dir).unwrap());
        assert!(result.is_err());
    }
}
