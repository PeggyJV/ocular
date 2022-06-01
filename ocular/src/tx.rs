use cosmrs::{Denom, AccountId};
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
pub struct SendTx {
    #[serde(default)]
    /// The bech32 address of the receiving account
    pub recipient: String,
    pub amount: u64,
    pub denom: String,
    pub metadata: TxMetadata,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SendTxToml {
    /// The name of the key in the client's keystore
    pub sender: String,
    pub transactions: Vec<SendTx>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiSendIO {
    pub address: String,
    pub coins: Vec<Coin>,
}

impl TryFrom<MultiSendIO> for cosmrs::bank::MultiSendIO {
    type Error = TxError;

    fn try_from(value: MultiSendIO) -> Result<cosmrs::bank::MultiSendIO, Self::Error> {
        cosmrs::bank::MultiSendIO::try_from(&value)
    }
}

impl TryFrom<&MultiSendIO> for cosmrs::bank::MultiSendIO {
    type Error = TxError;

    fn try_from(value: &MultiSendIO) -> Result<cosmrs::bank::MultiSendIO, Self::Error> {
        let id = bech32::decode(value.address.as_str())?;
        let bytes: Vec<u8> = id.1.iter().map(|b| b.to_u8()).collect();
        let id = AccountId::new(id.0.as_str(), &*bytes)?;

        Ok(cosmrs::bank::MultiSendIO {
            address: id,
            coins: value
                .coins
                .iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
