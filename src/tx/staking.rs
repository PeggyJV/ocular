//! Messages for managing stake delegations to validators
use std::str::FromStr;

use cosmrs::{Any, staking::{MsgBeginRedelegate, MsgUndelegate, MsgDelegate}, AccountId, tx::Msg, Denom};
use eyre::{Report, Result};

use super::{UnsignedTx, ModuleMsg};

/// Represents a [Staking module message](https://docs.cosmos.network/v0.45/modules/staking/03_messages.html)
///
/// MsgCreateValidator and MsgEditValidator are currently unimplimented
pub enum Staking<'m> {
    /// Delegate stake to a validator. Represents a [`MsgDelegate`]
    Delegate {
        delegator_address: &'m str,
        validator_address: &'m str,
        amount: u128,
        denom: &'m str,
    },
    /// Undelegate (remove) stake from a validator. Represents a [`MsgUndelegate`]
    Undelegate {
        delegator_address: &'m str,
        validator_address: &'m str,
        amount: u128,
        denom: &'m str,
    },
    /// Start a redelegation from one validator to another. Represents a [`MsgBeginRedelegate`]
    BeginRedelegate {
        delegator_address: &'m str,
        validator_src_address: &'m str,
        validator_dst_address: &'m str,
        amount: u128,
        denom: &'m str,
    },
}

impl ModuleMsg for Staking<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Staking::Delegate {
                delegator_address,
                validator_address,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };

                MsgDelegate {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_address: AccountId::from_str(validator_address)?,
                    amount,
                }
                .to_any()
            },
            Staking::Undelegate {
                delegator_address,
                validator_address,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };

                MsgUndelegate {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_address: AccountId::from_str(validator_address)?,
                    amount,
                }
                .to_any()
            },
            Staking::BeginRedelegate {
                delegator_address,
                validator_src_address,
                validator_dst_address,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };

                MsgBeginRedelegate {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_src_address: AccountId::from_str(validator_src_address)?,
                    validator_dst_address: AccountId::from_str(validator_dst_address)?,
                    amount,
                }
                .to_any()
            },
        }
    }

    /// Converts the message enum representation into an [`UnsignedTx`] containing the corresponding Msg
    fn into_tx(self) -> Result<UnsignedTx> {
        let mut tx = UnsignedTx::new();
        tx.add_msg(self.into_any()?);

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        Staking::Delegate {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Staking::Undelegate {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Staking::BeginRedelegate {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_src_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_dst_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();
    }
}
