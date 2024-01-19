//! Defines [`AccountInfo`], a private key wrapper used for signing and deriving addresses
use std::sync::Arc;

use bip32::{secp256k1::SecretKey, Language, Mnemonic};
use eyre::{Context, Result};
use k256::ecdsa::SigningKey;

use crate::cosmrs::{crypto::PublicKey, AccountId};

/// The derivation path commonly used by Cosmos chains for key generation
pub const COSMOS_BASE_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";

/// Represents a local account derived from a [`SigningKey`].
///
/// Note: Attempting a transaction with an account made from a newly generated key will fail as the account does not actually exist
/// on-chain yet.
#[derive(Clone)]
pub struct AccountInfo {
    public_key: PublicKey,
    private_key: Arc<SigningKey>,
}

impl AccountInfo {
    /// Constructs an [`AccountInfo`] from a mnemonic phrase and passphrase to salt the seed.
    /// If you don't wish to use a passphrase, set `passphrase` to `""`. Currently only supports
    /// 24 word phrases.
    pub fn from_mnemonic(phrase: &str, passphrase: &str) -> Result<Self> {
        let phrase = Mnemonic::new(phrase, Language::English)
            .wrap_err("failed to parse mnemonic phrase. be sure it is a 24 word phrase as this crate does not support fewer words.")?;
        let seed = phrase.to_seed(passphrase);
        let derivation_path = COSMOS_BASE_DERIVATION_PATH.parse::<bip32::DerivationPath>()?;
        let key = bip32::XPrv::derive_from_path(seed, &derivation_path)?;
        let key = SecretKey::from(key.private_key());
        let key = SigningKey::from_bytes(key.to_be_bytes().as_slice())?;

        Ok(AccountInfo::from(key))
    }

    /// Constructs an [`AccountInfo`] from a PEM-encoded, PKCS #8 key on disk.
    pub fn from_pem(path: &str) -> Result<Self> {
        use std::{fs, path::Path};

        use eyre::eyre;
        use pkcs8::DecodePrivateKey;

        let key_path = Path::new(path);
        if let Err(e) = Path::try_exists(key_path) {
            return Err(eyre!("{}", e));
        }

        let pem = fs::read_to_string(path)?;
        let key = SecretKey::from_pkcs8_pem(&pem)
            .map_err(|e| eyre!("error decoding key at {}: {}", path, e))?;
        let key = SigningKey::from_bytes(key.to_be_bytes().as_slice())?;

        Ok(AccountInfo::from(key))
    }

    /// Constructs an [`AccountInfo`] from an encrypted, PEM-encoded, PKCS #8 key on disk.
    pub fn from_encrypted_pem(path: &str, passphrase: &str) -> Result<Self> {
        use std::{fs, path::Path};

        use eyre::eyre;
        use pkcs8::DecodePrivateKey;

        let key_path = Path::new(path);
        if let Err(e) = Path::try_exists(key_path) {
            return Err(eyre!("{}", e));
        }

        let pem = fs::read_to_string(path)?;
        let key = SecretKey::from_pkcs8_encrypted_pem(&pem, passphrase)
            .map_err(|e| eyre!("error decoding encrypted key at {}: {}", path, e))?;
        let key = SigningKey::from_bytes(key.to_be_bytes().as_slice())?;

        Ok(AccountInfo::from(key))
    }

    /// Constructs an [`AccountInfo`] from an unencrypted DER-encoded, PKCS #8 formatted key on disk.
    pub fn from_der(path: &str) -> Result<Self> {
        use std::{fs, path::Path};

        use eyre::eyre;
        use pkcs8::DecodePrivateKey;

        let key_path = Path::new(path);
        if let Err(e) = Path::try_exists(key_path) {
            return Err(eyre!("{}", e));
        }

        let bytes = fs::read(path)?;
        let key = SecretKey::from_pkcs8_der(&bytes)
            .map_err(|e| eyre!("error decoding key at {}: {}", path, e))?;
        let key = SigningKey::from_bytes(key.to_be_bytes().as_slice())?;

        Ok(AccountInfo::from(key))
    }

    /// Constructs an [`AccountInfo`] from an encrypted, DER-encoded, PKCS #8 formatted key on disk.
    pub fn from_encrypted_der(path: &str, passphrase: &str) -> Result<Self> {
        use std::{fs, path::Path};

        use eyre::eyre;
        use pkcs8::DecodePrivateKey;

        let key_path = Path::new(path);
        if let Err(e) = Path::try_exists(key_path) {
            return Err(eyre!("{}", e));
        }

        let bytes = fs::read(path)?;
        let key = SecretKey::from_pkcs8_encrypted_der(&bytes, passphrase)
            .map_err(|e| eyre!("error decoding encrypted key at {}: {}", path, e))?;
        let key = SigningKey::from_bytes(key.to_be_bytes().as_slice())?;

        Ok(AccountInfo::from(key))
    }

    /// Gets the bech32 address with the given prefix
    pub fn address(&self, prefix: &str) -> Result<String> {
        Ok(self.id(prefix)?.as_ref().to_string())
    }

    /// Gets the [`AccountId`] representing the account
    pub fn id(&self, prefix: &str) -> Result<AccountId> {
        self.public_key.account_id(prefix)
    }

    /// Gets the account's public key
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    /// Gets a reference to the account's private key
    pub fn private_key(&self) -> &SigningKey {
        self.private_key.as_ref()
    }
}

impl From<SigningKey> for AccountInfo {
    fn from(value: SigningKey) -> Self {
        Self::from(Arc::new(value))
    }
}

impl From<Arc<SigningKey>> for AccountInfo {
    fn from(value: Arc<SigningKey>) -> Self {
        let private_key = value;
        let public_key = private_key.verifying_key();

        AccountInfo {
            private_key,
            public_key: public_key.into(),
        }
    }
}
