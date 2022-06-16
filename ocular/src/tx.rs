use cosmrs::{AccountId, Denom};
use serde::{Deserialize, Serialize};

use crate::error::TxError;

/// Metadata wrapper for transactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxMetadata {
    pub fee: Coin,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Payment {
    #[serde(default)]
    pub recipient: String,
    pub amount: u64,
    pub denom: String,
    pub metadata: Option<TxMetadata>,
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
