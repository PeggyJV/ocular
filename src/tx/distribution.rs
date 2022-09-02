//! Types for constructing Authz module Msgs
use std::str::FromStr;

use cosmrs::{Any, distribution::{MsgSetWithdrawAddress, MsgWithdrawDelegatorReward, MsgWithdrawValidatorCommission, MsgFundCommunityPool}, AccountId, tx::Msg, Denom};
use eyre::{Report, Result};

use super::{UnsignedTx, ModuleMsg};

/// Represents a [Distribution module message](https://docs.cosmos.network/v0.45/modules/distribution/04_messages.html)
pub enum Distribution<'m> {
    /// Represents a [`MsgSetWithdrawAddress`]
    SetWithdrawAddress {
        delegator_address: &'m str,
        withdraw_address: &'m str,
    },
    /// Represents a [`MsgWithdrawDelegatorReward`]
    WithdrawDelegatorReward {
        delegator_address: &'m str,
        validator_address: &'m str,
    },
    /// Represents a [`MsgWithdrawValidatorCommission`]
    WithdrawValidatorCommission {
        validator_address: &'m str,
    },
    /// Represents a [`MsgFundCommunityPool`]
    FundCommunityPool {
        amount: u128,
        denom: &'m str,
        depositor: &'m str,
    },
}

impl ModuleMsg for Distribution<'_> {
    type Error = Report;

    fn into_any(self) -> Result<Any> {
        match self {
            Distribution::SetWithdrawAddress {
                delegator_address,
                withdraw_address,
            } => {
                MsgSetWithdrawAddress {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    withdraw_address: AccountId::from_str(withdraw_address)?,
                }
                .to_any()
            },
            Distribution::WithdrawDelegatorReward {
                delegator_address,
                validator_address,
            } => {
                MsgWithdrawDelegatorReward {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_address: AccountId::from_str(validator_address)?,
                }
                .to_any()
            },
            Distribution::WithdrawValidatorCommission {
                validator_address,
            } => {
                MsgWithdrawValidatorCommission {
                    validator_address: AccountId::from_str(validator_address)?,
                }
                .to_any()
            },
            Distribution::FundCommunityPool {
                amount,
                denom,
                depositor,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };
                MsgFundCommunityPool {
                    depositor: AccountId::from_str(depositor)?,
                    amount: vec![amount],
                }
                .to_any()
            },
        }
    }

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
        Distribution::SetWithdrawAddress {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            withdraw_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();


        Distribution::WithdrawDelegatorReward {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();


        Distribution::WithdrawValidatorCommission {
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();


        Distribution::FundCommunityPool {
            amount: 0,
            denom: "uatom",
            depositor: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();
    }
}
