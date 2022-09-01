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

#[derive(Clone, Debug)]
pub enum Bank<'m> {
    Send {
        from: &'m str,
        to: &'m str,
        amount: u64,
        denom: &'m str,
    },
    MultiSend {
        inputs: Vec<MultiSendIo>,
        outputs: Vec<MultiSendIo>,
    },
}

impl ModuleMsg<'_> for Bank<'_> {
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
                let msg = MsgSend {
                    from_address: AccountId::from_str(from)?,
                    to_address: AccountId::from_str(to)?,
                    amount: vec![amount],
                };

                msg.to_any()
            }
            Bank::MultiSend { inputs, outputs } => {
                let msg = MsgMultiSend { inputs, outputs };

                msg.to_any()
            }
        }
    }

    fn into_tx(self) -> Result<UnsignedTx> {
        let msg = self.into_any()?;
        let mut unsigned = UnsignedTx::new();

        unsigned.msg(msg);

        Ok(unsigned)
    }
}
