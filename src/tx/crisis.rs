//! Message for checking invariants. To learn more, see [Implementing Invariants](https://docs.cosmos.network/master/building-modules/invariants.html)
//!
//! Since cosmrs doesn't currently have an [`cosmrs::tx::msg::Msg`] implementation for Crisis messages,
//! it's defined here.
use std::str::FromStr;

use cosmrs::{proto::traits::TypeUrl, tx::Msg, AccountId, Any};
use eyre::{Report, Result};
use prost::Message;

use super::{ModuleMsg, UnsignedTx};
use crate::cosmrs;

/// Represents a [Crisis module message](https://docs.cosmos.network/v0.45/modules/crisis/02_messages.html)
#[derive(Clone, Debug)]
pub enum Crisis<'m> {
    /// Verify a given invariant for a given module. Represents a [`MsgVerifyInvariant`]
    VerifyInvariant {
        sender: &'m str,
        module_name: &'m str,
        route: &'m str,
    },
}

impl ModuleMsg for Crisis<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Crisis::VerifyInvariant {
                sender,
                module_name,
                route,
            } => MsgVerifyInvariant {
                sender: AccountId::from_str(sender)?,
                invariant_module_name: module_name.to_string(),
                invariant_route: route.to_string(),
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

// We implement cosmrs::tx::Msg for crisis Msgs because they're not in cosmrs
#[derive(Debug, Default)]
pub struct WrappedMsgVerifyInvariant {
    inner: cosmrs::proto::cosmos::crisis::v1beta1::MsgVerifyInvariant,
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
    const TYPE_URL: &'static str = "/cosmos.crisis.v1beta1.MsgVerifyInvariant";
}

/// Represents a message to verify a given invariant of a given module.
#[derive(Clone, Debug)]
pub struct MsgVerifyInvariant {
    /// Sender's address.
    pub sender: AccountId,

    /// Name of the module that contains the invariant
    pub invariant_module_name: String,

    /// The route of the invariant as defined in the chain's invariant registry.
    pub invariant_route: String,
}

impl Msg for MsgVerifyInvariant {
    type Proto = WrappedMsgVerifyInvariant;
}

impl TryFrom<WrappedMsgVerifyInvariant> for MsgVerifyInvariant {
    type Error = Report;

    fn try_from(proto: WrappedMsgVerifyInvariant) -> Result<MsgVerifyInvariant> {
        MsgVerifyInvariant::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgVerifyInvariant> for MsgVerifyInvariant {
    type Error = Report;

    fn try_from(proto: &WrappedMsgVerifyInvariant) -> Result<MsgVerifyInvariant> {
        Ok(MsgVerifyInvariant {
            sender: AccountId::from_str(&proto.inner.sender)?,
            invariant_module_name: proto.inner.invariant_module_name.clone(),
            invariant_route: proto.inner.invariant_route.clone(),
        })
    }
}

impl From<MsgVerifyInvariant> for WrappedMsgVerifyInvariant {
    fn from(coin: MsgVerifyInvariant) -> WrappedMsgVerifyInvariant {
        WrappedMsgVerifyInvariant::from(&coin)
    }
}

impl From<&MsgVerifyInvariant> for WrappedMsgVerifyInvariant {
    fn from(msg: &MsgVerifyInvariant) -> WrappedMsgVerifyInvariant {
        WrappedMsgVerifyInvariant {
            inner: cosmrs::proto::cosmos::crisis::v1beta1::MsgVerifyInvariant {
                sender: msg.sender.to_string(),
                invariant_module_name: msg.invariant_module_name.clone(),
                invariant_route: msg.invariant_route.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        Crisis::VerifyInvariant {
            sender: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            module_name: "",
            route: "",
        }
        .into_tx()
        .unwrap();
    }
}
