use cosmrs::{AccountId, Denom};
use serde::{Deserialize, Serialize};

use crate::error::TxError;

pub use prost_types::Any;

/// Metadata wrapper for transactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxMetadata {
    pub fee: Coin,
    pub fee_payer: Option<AccountId>,
    pub fee_granter: Option<AccountId>,
    pub gas_limit: u64,
    pub timeout_height: u32,
    #[serde(default)]
    pub memo: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Coin {
    pub amount: u64,
    pub denom: String,
}

impl From<Coin> for cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
    fn from(coin: Coin) -> Self {
        cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            amount: coin.amount.to_string(),
            denom: coin.denom,
        }
    }
}

impl TryFrom<Coin> for cosmrs::Coin {
    type Error = TxError;

    fn try_from(coin: Coin) -> Result<cosmrs::Coin, Self::Error> {
        cosmrs::Coin::try_from(&coin)
    }
}

impl TryFrom<&Coin> for cosmrs::Coin {
    type Error = TxError;

    fn try_from(coin: &Coin) -> Result<cosmrs::Coin, Self::Error> {
        Ok(cosmrs::Coin {
            denom: coin.denom.parse::<Denom>()?,
            amount: coin.amount.into(),
        })
    }
}

impl TryFrom<cosmos_sdk_proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = TxError;

    fn try_from(coin: cosmos_sdk_proto::cosmos::base::v1beta1::Coin) -> Result<Coin, Self::Error> {
        Coin::try_from(&coin)
    }
}

impl TryFrom<&cosmos_sdk_proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = TxError;

    fn try_from(coin: &cosmos_sdk_proto::cosmos::base::v1beta1::Coin) -> Result<Coin, Self::Error> {
        Ok(Coin {
            denom: coin.denom.clone(),
            amount: coin.amount.parse()?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Payment {
    pub recipient: String,
    pub amount: u64,
    pub denom: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PaymentsToml {
    pub sender_key_name: String,
    pub grantee_key_name: Option<String>,
    pub fee_granter: Option<String>,
    pub fee_payer: Option<String>,
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiSendIo {
    pub address: String,
    pub coins: Vec<Coin>,
}

impl TryFrom<MultiSendIo> for cosmrs::bank::MultiSendIo {
    type Error = TxError;

    fn try_from(value: MultiSendIo) -> Result<cosmrs::bank::MultiSendIo, Self::Error> {
        cosmrs::bank::MultiSendIo::try_from(&value)
    }
}

impl TryFrom<&MultiSendIo> for cosmrs::bank::MultiSendIo {
    type Error = TxError;

    fn try_from(value: &MultiSendIo) -> Result<cosmrs::bank::MultiSendIo, Self::Error> {
        Ok(cosmrs::bank::MultiSendIo {
            address: value.address.as_str().parse::<AccountId>()?,
            coins: value
                .coins
                .iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_coin() {
        let coin = Coin {
            amount: 100,
            denom: "utest".to_string(),
        };

        cosmrs::Coin::try_from(&coin).unwrap();
        cosmrs::Coin::try_from(coin).unwrap();
    }
}
