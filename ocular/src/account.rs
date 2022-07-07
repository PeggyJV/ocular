use std::sync::Arc;

use cosmrs::{crypto::secp256k1::SigningKey, crypto::PublicKey, AccountId};

use crate::error::AccountError;

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
    pub fn address(&self, prefix: &str) -> String {
        self.id(prefix).as_ref().to_string()
    }

    pub fn id(&self, prefix: &str) -> AccountId {
        self.public_key
            .account_id(prefix)
            .expect("failed to derive account id from public key. if this panic ever occurs there is a bug with AccountInfo construction.")
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn private_key(&self) -> &SigningKey {
        self.private_key.as_ref()
    }
}

impl TryFrom<SigningKey> for AccountInfo {
    type Error = AccountError;

    fn try_from(value: SigningKey) -> Result<Self, Self::Error> {
        Self::try_from(Arc::new(value))
    }
}

impl TryFrom<Arc<SigningKey>> for AccountInfo {
    type Error = AccountError;

    fn try_from(value: Arc<SigningKey>) -> Result<Self, Self::Error> {
        let private_key = value;
        let public_key = private_key.public_key();

        // By doing this check here, we can assert that self.id() and self.address() will not error (so long as all constructors make this check)
        if public_key.account_id("testprefix").is_err() {
            return Err(AccountError::InvalidPublicKey(
                "unable to derive AccountId from key".to_string(),
            ));
        }

        Ok(AccountInfo {
            private_key,
            public_key,
        })
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

impl TryFrom<cosmrs::proto::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = AccountError;

    fn try_from(
        account: cosmrs::proto::cosmos::auth::v1beta1::BaseAccount,
    ) -> Result<BaseAccount, Self::Error> {
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
