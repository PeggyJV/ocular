#![allow(dead_code)]
//! Types for constructing Authz module Msgs
//!
//! Since cosmrs doesn't currently have [`cosmrs::tx::msg::Msg`] implementations for Authz messages,
//! they are defined here as well.
use std::str::FromStr;

use eyre::{eyre, Report, Result};
use cosmrs::{proto::{cosmos::authz::v1beta1::Grant, traits::TypeUrl}, Any, AccountId, tx::Msg};
use prost::Message;

use super::{ModuleMsg, UnsignedTx};

#[derive(Clone, Debug)]
pub enum Authz<'m> {
    Grant {
        granter: &'m str,
        grantee: &'m str,
        grant: Grant,
    },
    Revoke {
        granter: &'m str,
        grantee: &'m str,
        msg_type_url: &'m str,
    },
    Exec {
        grantee: &'m str,
        msgs: Vec<Any>,
    },
}

impl ModuleMsg for Authz<'_> {
    type Error = Report;

    fn into_any(self) -> Result<Any> {
        match self {
            Authz::Grant {
                granter,
                grantee,
                grant,
            } => {
                MsgGrant {
                    granter: AccountId::from_str(granter)?,
                    grantee: AccountId::from_str(grantee)?,
                    grant: grant,
                }
                .to_any()
            },
            Authz::Revoke {
                granter,
                grantee,
                msg_type_url,
            } => {
                MsgRevoke {
                    granter: AccountId::from_str(granter)?,
                    grantee: AccountId::from_str(grantee)?,
                    msg_type_url: msg_type_url.to_string(),
                }
                .to_any()
            },
            Authz::Exec {
                grantee,
                msgs,
            } => {
                MsgExec {
                    grantee: AccountId::from_str(grantee)?,
                    msgs,
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

// Implement cosmrs::tx::Msg for authz Msgs because it's not in cosmrs
#[derive(Debug)]
pub struct WrappedMsgGrant {
    inner: cosmrs::proto::cosmos::authz::v1beta1::MsgGrant,
}

impl Message for WrappedMsgGrant {
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

impl Default for WrappedMsgGrant {
    fn default() -> Self {
        WrappedMsgGrant {
            inner: cosmrs::proto::cosmos::authz::v1beta1::MsgGrant::default()
        }
    }
}

impl TypeUrl for WrappedMsgGrant {
    const TYPE_URL: &'static str = "/cosmos.authz.v1beta1.MsgGrant";
}

/// MsgGrant represents a message to grant authorization to execute a Msg from `granter` to `grantee`
#[derive(Clone, Debug)]
pub struct MsgGrant {
    /// Sender's address.
    pub granter: AccountId,

    /// Recipient's address.
    pub grantee: AccountId,

    /// Amount to send
    pub grant: Grant,
}

impl Msg for MsgGrant {
    type Proto = WrappedMsgGrant;
}

impl TryFrom<WrappedMsgGrant> for MsgGrant {
    type Error = Report;

    fn try_from(proto: WrappedMsgGrant) -> Result<MsgGrant> {
        MsgGrant::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgGrant> for MsgGrant {
    type Error = Report;

    fn try_from(proto: &WrappedMsgGrant) -> Result<MsgGrant> {
        Ok(MsgGrant {
            granter: AccountId::from_str(&proto.inner.granter)?,
            grantee: AccountId::from_str(&proto.inner.grantee)?,
            grant: proto.inner.grant.clone().ok_or(eyre!("grant cannot be empty"))?
        })
    }
}

impl From<MsgGrant> for WrappedMsgGrant {
    fn from(coin: MsgGrant) -> WrappedMsgGrant {
        WrappedMsgGrant::from(&coin)
    }
}

impl From<&MsgGrant> for WrappedMsgGrant {
    fn from(msg: &MsgGrant) -> WrappedMsgGrant {
        WrappedMsgGrant {
            inner: cosmrs::proto::cosmos::authz::v1beta1::MsgGrant {
                granter: msg.granter.to_string(),
                grantee: msg.grantee.to_string(),
                grant: Some(msg.grant.clone())
            }
        }
    }
}

#[derive(Debug)]
pub struct WrappedMsgRevoke {
    inner: cosmrs::proto::cosmos::authz::v1beta1::MsgRevoke,
}

impl Message for WrappedMsgRevoke {
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

impl Default for WrappedMsgRevoke {
    fn default() -> Self {
        WrappedMsgRevoke {
            inner: cosmrs::proto::cosmos::authz::v1beta1::MsgRevoke::default()
        }
    }
}

impl TypeUrl for WrappedMsgRevoke {
    const TYPE_URL: &'static str = "/cosmos.authz.v1beta1.MsgRevoke";
}

/// MsgRevoke represents a message to revoke a [`Grant`] from `granter` to `grantee`
#[derive(Clone, Debug)]
pub struct MsgRevoke {
    /// Granter's address.
    pub granter: AccountId,

    /// Grantee's address.
    pub grantee: AccountId,

    pub msg_type_url: String,
}

impl Msg for MsgRevoke {
    type Proto = WrappedMsgRevoke;
}

impl TryFrom<WrappedMsgRevoke> for MsgRevoke {
    type Error = Report;

    fn try_from(proto: WrappedMsgRevoke) -> Result<MsgRevoke> {
        MsgRevoke::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgRevoke> for MsgRevoke {
    type Error = Report;

    fn try_from(proto: &WrappedMsgRevoke) -> Result<MsgRevoke> {
        Ok(MsgRevoke {
            granter: AccountId::from_str(&proto.inner.granter)?,
            grantee: AccountId::from_str(&proto.inner.grantee)?,
            msg_type_url: proto.inner.msg_type_url.clone(),
        })
    }
}

impl From<MsgRevoke> for WrappedMsgRevoke {
    fn from(coin: MsgRevoke) -> WrappedMsgRevoke {
        WrappedMsgRevoke::from(&coin)
    }
}

impl From<&MsgRevoke> for WrappedMsgRevoke {
    fn from(msg: &MsgRevoke) -> WrappedMsgRevoke {
        WrappedMsgRevoke {
            inner: cosmrs::proto::cosmos::authz::v1beta1::MsgRevoke {
                granter: msg.granter.to_string(),
                grantee: msg.grantee.to_string(),
                msg_type_url: msg.msg_type_url.clone()
            }
        }
    }
}

#[derive(Debug)]
pub struct WrappedMsgExec {
    inner: cosmrs::proto::cosmos::authz::v1beta1::MsgExec,
}

impl Message for WrappedMsgExec {
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

impl Default for WrappedMsgExec {
    fn default() -> Self {
        WrappedMsgExec {
            inner: cosmrs::proto::cosmos::authz::v1beta1::MsgExec::default()
        }
    }
}

impl TypeUrl for WrappedMsgExec {
    const TYPE_URL: &'static str = "/cosmos.authz.v1beta1.MsgExec";
}

/// MsgExec represents a message to execute a tx on behalf of another account
#[derive(Clone, Debug)]
pub struct MsgExec {
    /// Grantee's address.
    pub grantee: AccountId,

    /// Msgs to execute on behalf of a granter
    pub msgs: Vec<Any>,
}

impl Msg for MsgExec {
    type Proto = WrappedMsgExec;
}

impl TryFrom<WrappedMsgExec> for MsgExec {
    type Error = Report;

    fn try_from(proto: WrappedMsgExec) -> Result<MsgExec> {
        MsgExec::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgExec> for MsgExec {
    type Error = Report;

    fn try_from(proto: &WrappedMsgExec) -> Result<MsgExec> {
        Ok(MsgExec {
            grantee: AccountId::from_str(&proto.inner.grantee)?,
            msgs: proto.inner.msgs.clone(),
        })
    }
}

impl From<MsgExec> for WrappedMsgExec {
    fn from(coin: MsgExec) -> WrappedMsgExec {
        WrappedMsgExec::from(&coin)
    }
}

impl From<&MsgExec> for WrappedMsgExec {
    fn from(msg: &MsgExec) -> WrappedMsgExec {
        WrappedMsgExec {
            inner: cosmrs::proto::cosmos::authz::v1beta1::MsgExec {
                grantee: msg.grantee.to_string(),
                msgs: msg.msgs.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        Authz::Grant {
            granter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            grantee: "cosmos1n6j7gnld9yxfyh6tflxhjjmt404zruuaf73t08",
            grant: Grant {
                authorization: None,
                expiration: None,
            }
        }
        .into_tx()
        .unwrap();

        Authz::Revoke {
            granter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            grantee: "cosmos1n6j7gnld9yxfyh6tflxhjjmt404zruuaf73t08",
            msg_type_url: "",
        }
        .into_tx()
        .unwrap();


        Authz::Exec {
            grantee: "cosmos1n6j7gnld9yxfyh6tflxhjjmt404zruuaf73t08",
            msgs: vec![]
        }
        .into_tx()
        .unwrap();
    }
}
