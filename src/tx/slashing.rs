//! Message for unjailing a validator
//!
//! Since cosmrs doesn't currently have a [`cosmrs::tx::msg::Msg`] implementation for Slashing messages,
//! it's defined here.
use std::str::FromStr;

use cosmrs::{proto::traits::TypeUrl, tx::Msg, AccountId, Any};
use eyre::{Report, Result};
use prost::Message;

use super::{ModuleMsg, UnsignedTx};
use crate::cosmrs;

/// Represents a [Slashing module message](https://docs.cosmos.network/v0.45/modules/slashing/03_messages.html)
#[derive(Clone, Debug)]
pub enum Slashing<'m> {
    /// Unjail a jailed validator. Represents a [`MsgUnjail`]
    Unjail { validator_address: &'m str },
}

impl ModuleMsg for Slashing<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Slashing::Unjail { validator_address } => MsgUnjail {
                validator_addr: AccountId::from_str(validator_address)?,
            }
            .to_any(),
        }
    }

    /// Converts the message enum representation into an [`UnsignedTx`] containing the corresponding Msg
    fn into_tx(self) -> Result<UnsignedTx> {
        let mut tx = UnsignedTx::new();
        tx.add_msg(self.into_any()?);

        Ok(tx)
    }
}

// We implement cosmrs::tx::Msg for slashing Msgs because they're not in cosmrs
#[derive(Debug, Default)]
pub struct WrappedMsgVerifyInvariant {
    inner: cosmrs::proto::cosmos::slashing::v1beta1::MsgUnjail,
}

impl Message for WrappedMsgVerifyInvariant {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: prost::bytes::BufMut,
        Self: Sized,
    {
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
        Self: Sized,
    {
        self.inner.merge_field::<B>(tag, wire_type, buf, ctx)
    }

    fn encoded_len(&self) -> usize {
        self.inner.encoded_len()
    }

    fn clear(&mut self) {
        self.inner.clear()
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
            },
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
