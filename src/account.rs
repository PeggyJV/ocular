//! Types pertaining to auth accounts
#[cfg(feature = "signing")]
use std::sync::Arc;

use eyre::{Report, Result};

use crate::cosmrs::crypto::PublicKey;
#[cfg(feature = "signing")]
use crate::cosmrs::{crypto::secp256k1::SigningKey, AccountId};

/// Represents a local account derived from a [`SigningKey`].
///
/// Note: Attempting a transaction with an account made from a newly generated key will fail as the account does not actually exist
/// on-chain yet.
#[cfg(feature = "signing")]
#[derive(Clone)]
pub struct AccountInfo {
    public_key: PublicKey,
    private_key: Arc<SigningKey>,
}

#[cfg(feature = "signing")]
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

#[cfg(feature = "signing")]
impl From<SigningKey> for AccountInfo {
    fn from(value: SigningKey) -> Self {
        Self::from(Arc::new(value))
    }
}

#[cfg(feature = "signing")]
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

/// Used for converting the BaseAccount type in cosmos_sdk_proto to something with concrete field types
#[derive(Clone, Debug)]
pub struct BaseAccount {
    pub address: String,
    // public key may not be present on chain
    pub pub_key: Option<PublicKey>,
    pub account_number: u64,
    pub sequence: u64,
}

// TO-DO: Handle public keys with type URL /cosmos.crypto.multisig.LegacyAminoPubKey
impl TryFrom<cosmrs::proto::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = Report;

    fn try_from(account: cosmrs::proto::cosmos::auth::v1beta1::BaseAccount) -> Result<BaseAccount> {
        let pub_key = match account.pub_key {
            Some(k) => Some(PublicKey::try_from(k)?),
            None => None,
        };

        Ok(BaseAccount {
            address: account.address,
            pub_key,
            account_number: account.account_number,
            sequence: account.sequence,
        })
    }
}
