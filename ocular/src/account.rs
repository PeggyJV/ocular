use cosmrs::{crypto::secp256k1::SigningKey, crypto::PublicKey, AccountId};

use crate::error::AccountError;

///  Type to hold all information around an account.
pub struct AccountInfo {
    pub id: AccountId,
    pub public_key: PublicKey,
    pub private_key: SigningKey,
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
