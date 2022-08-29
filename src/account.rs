#![cfg(feature = "tx")]
//! Types pertaining to auth accounts
use std::sync::Arc;

use eyre::Result;

use crate::cosmrs::{AccountId, crypto::PublicKey};

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
