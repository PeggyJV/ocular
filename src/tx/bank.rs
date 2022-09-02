#![allow(dead_code)]
//! Types for constructing Bank module Msgs
use std::str::FromStr;

use eyre::{Report, Result};

use crate::cosmrs::{
    self,
    bank::{MsgMultiSend, MsgSend, MultiSendIo},
    tx::Msg,
    AccountId, Any, Denom,
};

use super::{ModuleMsg, UnsignedTx};

/// Represents a [Bank module message](https://docs.cosmos.network/v0.45/modules/bank/03_messages.html)
#[derive(Clone, Debug)]
pub enum Bank<'m> {
    /// Represents a [`MsgSend`]
    Send {
        from: &'m str,
        to: &'m str,
        amount: u64,
        denom: &'m str,
    },
    /// Represents a [`MsgMultiSend`]
    MultiSend {
        inputs: Vec<MultiSendIo>,
        outputs: Vec<MultiSendIo>,
    },
}

impl ModuleMsg for Bank<'_> {
    type Error = Report;

    fn into_any(self) -> Result<Any> {
        match self {
            Bank::Send {
                from,
                to,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };
                MsgSend {
                    from_address: AccountId::from_str(from)?,
                    to_address: AccountId::from_str(to)?,
                    amount: vec![amount],
                }
                .to_any()
            }
            Bank::MultiSend { inputs, outputs } => {
                MsgMultiSend { inputs, outputs }.to_any()
            }
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
        Bank::Send {
            to: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            from: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom"
        }
        .into_tx()
        .unwrap();

        Bank::MultiSend {
            inputs: vec![],
            outputs: vec![],
        }
        .into_tx()
        .unwrap();
    }
}

