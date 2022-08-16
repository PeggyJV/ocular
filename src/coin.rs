//! A simple coin type convertable from [`cosmrs::Coin`] and the Cosmos SDK proto coin types.
use cosmrs::Denom;
use eyre::{Report, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Coin {
    pub amount: u128,
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
    type Error = Report;

    fn try_from(coin: Coin) -> Result<cosmrs::Coin, Self::Error> {
        cosmrs::Coin::try_from(&coin)
    }
}

impl TryFrom<&Coin> for cosmrs::Coin {
    type Error = Report;

    fn try_from(coin: &Coin) -> Result<cosmrs::Coin, Self::Error> {
        Ok(cosmrs::Coin {
            denom: coin.denom.parse::<Denom>()?,
            amount: (coin.amount as u64).into(),
        })
    }
}

impl TryFrom<cosmos_sdk_proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = Report;

    fn try_from(coin: cosmos_sdk_proto::cosmos::base::v1beta1::Coin) -> Result<Coin, Self::Error> {
        Coin::try_from(&coin)
    }
}

impl TryFrom<&cosmos_sdk_proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = Report;

    fn try_from(coin: &cosmos_sdk_proto::cosmos::base::v1beta1::Coin) -> Result<Coin, Self::Error> {
        Ok(Coin {
            denom: coin.denom.clone(),
            amount: coin.amount.parse()?,
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
