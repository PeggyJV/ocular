#![cfg(feature = "tx")]
//! Defines [`AccountInfo`], a private key wrapper used for signing and deriving addresses
use std::sync::Arc;

use cosmrs::bip32::Language;
use eyre::Result;

use crate::cosmrs::{
    bip32::{secp256k1::SecretKey, Mnemonic},
    crypto::PublicKey,
    AccountId,
};

pub const COSMOS_BASE_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";

/// Represents a local account derived from a [`SigningKey`].
///
/// Note: Attempting a transaction with an account made from a newly generated key will fail as the account does not actually exist
/// on-chain yet.
use crate::cosmrs::crypto::secp256k1::SigningKey;

#[derive(Clone)]
pub struct AccountInfo {
    public_key: PublicKey,
    private_key: Arc<SigningKey>,
}

impl AccountInfo {
    pub fn address(&self, prefix: &str) -> Result<String> {
        Ok(self.id(prefix)?.as_ref().to_string())
    }

    pub fn id(&self, prefix: &str) -> Result<AccountId> {
        self.public_key.account_id(prefix)
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn private_key(&self) -> &SigningKey {
        self.private_key.as_ref()
    }

    /// Constructs an [`AccountInfo`] from a mnemonic phrase and passphrase to salt the seed.
    /// If you don't wish to use a passphrase, set `passphrase` to `""`. Currently only supports
    /// 24 word phrases.
    pub fn from_mnemonic(phrase: &str, passphrase: &str) -> Result<Self> {
        let phrase = Mnemonic::new(phrase, Language::English)?;
        let seed = phrase.to_seed(passphrase);
        let derivation_path =
            COSMOS_BASE_DERIVATION_PATH.parse::<cosmrs::bip32::DerivationPath>()?;
        let key = cosmrs::bip32::XPrv::derive_from_path(seed, &derivation_path)?;
        let key = SecretKey::from(key.private_key());
        let key = SigningKey::from_bytes(&key.to_be_bytes())?;

        Ok(AccountInfo::from(key))
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
        let public_key = private_key.public_key();

        AccountInfo {
            private_key,
            public_key,
        }
    }
}
