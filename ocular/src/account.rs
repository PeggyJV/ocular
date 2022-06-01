use cosmrs::{crypto::secp256k1::SigningKey, crypto::PublicKey, AccountId};

use crate::error::AccountError;

///  Type to hold all information around an account.
pub struct Account {
    pub id: AccountId,
    pub public_key: PublicKey,
    pub private_key: SigningKey,
}

/// Used for converting the BaseAccount type in cosmos_sdk_proto to something with concrete field types
#[derive(Clone, Debug)]
pub struct BaseAccount {
    pub address: String,
    pub pub_key: PublicKey,
    pub account_number: u64,
    pub sequence: u64,
}

impl TryFrom<cosmrs::proto::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = AccountError;

    fn try_from(
        account: cosmrs::proto::cosmos::auth::v1beta1::BaseAccount,
    ) -> Result<BaseAccount, Self::Error> {
        let pub_key = match account.pub_key {
            Some(k) => k,
            None => return Err(AccountError::Empty("field pub_key is None".into())),
        };
        let pub_key = PublicKey::try_from(pub_key)?;

        Ok(BaseAccount {
            address: account.address,
            pub_key,
            account_number: account.account_number,
            sequence: account.sequence,
        })
    }
}
