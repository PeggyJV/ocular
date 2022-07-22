use std::sync::Arc;

use bip32::{Mnemonic, PrivateKey};
use k256::SecretKey;
use pkcs8::{DecodePrivateKey, EncodePrivateKey};
use rand_core::OsRng;

use crate::{error::AccountError, keyring::COSMOS_BASE_DERIVATION_PATH};

/// Represents a bech32 account identifier
pub use cosmrs::AccountId;
pub use cosmrs::crypto::{PublicKey, secp256k1::SigningKey};

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
    /// Constructor that generates a new private key
    pub fn new(password: &str) -> AccountInfo {
        let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
        let seed = mnemonic.to_seed(password);
        let derivation_path = COSMOS_BASE_DERIVATION_PATH;
        let derivation_path = derivation_path
            .parse::<bip32::DerivationPath>()
            .expect("Could not parse derivation path.");
        let extended_signing_key =
            bip32::XPrv::derive_from_path(seed, &derivation_path).expect("Could not derive key.");
        let signing_key = k256::SecretKey::from(extended_signing_key.private_key());
        let encoded_key = signing_key
            .to_pkcs8_der()
            .expect("Could not PKCS8 encode private key");
        let decoded_private_key: SecretKey = DecodePrivateKey::from_pkcs8_doc(&encoded_key)
            .expect("Could not decode private key document.");
        let private_key = SigningKey::from_bytes(&decoded_private_key.to_bytes())
            .expect("Could not create signing key.");

        AccountInfo::from(private_key)
    }

    pub fn address(&self, prefix: &str) -> Result<String, AccountError> {
        Ok(self.id(prefix)?.as_ref().to_string())
    }

    pub fn id(&self, prefix: &str) -> Result<AccountId, AccountError> {
        Ok(self.public_key.account_id(prefix)?)
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
