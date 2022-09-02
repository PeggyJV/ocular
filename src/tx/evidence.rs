//! Types for constructing Evidence module Msgs
//!
//! Since cosmrs doesn't currently have [`cosmrs::tx::msg::Msg`] implementations for Evidence messages,
//! they are defined here as well.
use std::str::FromStr;

use cosmrs::{AccountId, tx::Msg, proto::traits::TypeUrl, Any};
use eyre::{eyre, Report, Result};
use prost::Message;

use crate::cosmrs;
use super::{ModuleMsg, UnsignedTx};

/// Represents a [Evidence module message](https://docs.cosmos.network/v0.45/modules/evidence/03_messages.html)
#[derive(Clone, Debug)]
pub enum Evidence<'m> {
    /// Represents a [`MsgSubmitEvidence`]
    SubmitEvidence {
        submitter: &'m str,
        evidence: Any,
    },
}

impl ModuleMsg for Evidence<'_> {
    type Error = Report;

    fn into_any(self) -> Result<Any> {
        match self {
            Evidence::SubmitEvidence {
                submitter,
                evidence,
            } => {
                MsgSubmitEvidence {
                    submitter: AccountId::from_str(submitter)?,
                    evidence,
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
pub struct WrappedMsgSubmitEvidence {
    inner: cosmrs::proto::cosmos::evidence::v1beta1::MsgSubmitEvidence,
}

impl Message for WrappedMsgSubmitEvidence {
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

impl Default for WrappedMsgSubmitEvidence {
    fn default() -> Self {
        WrappedMsgSubmitEvidence {
            inner: cosmrs::proto::cosmos::evidence::v1beta1::MsgSubmitEvidence::default()
        }
    }
}

impl TypeUrl for WrappedMsgSubmitEvidence {
    const TYPE_URL: &'static str = "/cosmos.evidence.v1beta1.MsgSubmitEvidence";
}

/// MsgSubmitEvidence represents a message to send coins from one account to another.
#[derive(Clone, Debug)]
pub struct MsgSubmitEvidence {
    /// Submitter's address.
    pub submitter: AccountId,
    /// Evidence to submit
    pub evidence: Any,
}

impl Msg for MsgSubmitEvidence {
    type Proto = WrappedMsgSubmitEvidence;
}

impl TryFrom<WrappedMsgSubmitEvidence> for MsgSubmitEvidence {
    type Error = Report;

    fn try_from(proto: WrappedMsgSubmitEvidence) -> Result<MsgSubmitEvidence> {
        MsgSubmitEvidence::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgSubmitEvidence> for MsgSubmitEvidence {
    type Error = Report;

    fn try_from(proto: &WrappedMsgSubmitEvidence) -> Result<MsgSubmitEvidence> {
        Ok(MsgSubmitEvidence {
            submitter: AccountId::from_str(&proto.inner.submitter)?,
            evidence: proto.inner.evidence.to_owned().ok_or(eyre!("evidence cannot be empty"))?,
        })
    }
}

impl From<MsgSubmitEvidence> for WrappedMsgSubmitEvidence {
    fn from(coin: MsgSubmitEvidence) -> WrappedMsgSubmitEvidence {
        WrappedMsgSubmitEvidence::from(&coin)
    }
}

impl From<&MsgSubmitEvidence> for WrappedMsgSubmitEvidence {
    fn from(msg: &MsgSubmitEvidence) -> WrappedMsgSubmitEvidence {
        WrappedMsgSubmitEvidence {
            inner: cosmrs::proto::cosmos::evidence::v1beta1::MsgSubmitEvidence {
                submitter: msg.submitter.to_string(),
                evidence: Some(msg.evidence.to_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        Evidence::SubmitEvidence {
            submitter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            evidence: Any::default(),
        }
        .into_tx()
        .unwrap();
    }
}
