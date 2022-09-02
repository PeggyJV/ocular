//! Types for constructing Slashing module Msgs
//!
//! Since cosmrs doesn't currently have [`cosmrs::tx::msg::Msg`] implementations for Slashing messages,
//! they are defined here as well.
use std::str::FromStr;

use cosmrs::{AccountId, tx::Msg, proto::traits::TypeUrl, Any};
use eyre::{Report, Result};
use prost::Message;

use crate::cosmrs;
use super::{ModuleMsg, UnsignedTx};

/// Represents a [Slashing module message](https://docs.cosmos.network/v0.45/modules/slashing/03_messages.html)
#[derive(Clone, Debug)]
pub enum Slashing<'m> {
    /// Represents a [`MsgUnjail`]
    Unjail {
        validator_address: &'m str,
    },
}

impl ModuleMsg for Slashing<'_> {
    type Error = Report;

    fn into_any(self) -> Result<Any> {
        match self {
            Slashing::Unjail { validator_address } => {
                MsgUnjail {
                    validator_addr: AccountId::from_str(validator_address)?,
                }
                .to_any()
            }
        }
    }

    fn into_tx(self) -> Result<UnsignedTx> {
        let mut tx = UnsignedTx::new();
        tx.add_msg(self.into_any()?);

        Ok(tx)
    }
}

#[derive(Debug)]
pub struct WrappedMsgVerifyInvariant {
    inner: cosmrs::proto::cosmos::slashing::v1beta1::MsgUnjail,
}

impl Message for WrappedMsgVerifyInvariant {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: prost::bytes::BufMut,
        Self: Sized {
        self.inner.encode_raw::<B>(buf);
    }

    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut B,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError>
    where
        B: prost::bytes::Buf,
        Self: Sized {
        self.inner.merge_field::<B>(tag, wire_type, buf, ctx)
    }

    fn encoded_len(&self) -> usize {
        self.inner.encoded_len()
    }

    fn clear(&mut self) {
        self.inner.clear()
    }
}

impl Default for WrappedMsgVerifyInvariant {
    fn default() -> Self {
        WrappedMsgVerifyInvariant {
            inner: cosmrs::proto::cosmos::slashing::v1beta1::MsgUnjail::default()
        }
    }
}

impl TypeUrl for WrappedMsgVerifyInvariant {
    const TYPE_URL: &'static str = "/cosmos.slashing.v1beta1.MsgUnjail";
}

/// MsgUnjail represents a message to send coins from one account to another.
#[derive(Clone, Debug)]
pub struct MsgUnjail {
    /// Address of the validator to unjail
    pub validator_addr: AccountId,
}

impl Msg for MsgUnjail {
    type Proto = WrappedMsgVerifyInvariant;
}

impl TryFrom<WrappedMsgVerifyInvariant> for MsgUnjail {
    type Error = Report;

    fn try_from(proto: WrappedMsgVerifyInvariant) -> Result<MsgUnjail> {
        MsgUnjail::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgVerifyInvariant> for MsgUnjail {
    type Error = Report;

    fn try_from(proto: &WrappedMsgVerifyInvariant) -> Result<MsgUnjail> {
        Ok(MsgUnjail {
            validator_addr: AccountId::from_str(&proto.inner.validator_addr)?,
        })
    }
}

impl From<MsgUnjail> for WrappedMsgVerifyInvariant {
    fn from(coin: MsgUnjail) -> WrappedMsgVerifyInvariant {
        WrappedMsgVerifyInvariant::from(&coin)
    }
}

impl From<&MsgUnjail> for WrappedMsgVerifyInvariant {
    fn from(msg: &MsgUnjail) -> WrappedMsgVerifyInvariant {
        WrappedMsgVerifyInvariant {
            inner: cosmrs::proto::cosmos::slashing::v1beta1::MsgUnjail {
                validator_addr: msg.validator_addr.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        Slashing::Unjail {
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();
    }
}

