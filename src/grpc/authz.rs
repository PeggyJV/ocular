//! Query methods for the [Authz module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/authz/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`AuthzQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{tx::Msg, AccountId, Any};
use eyre::{eyre, Context, Report, Result};
use prost::{Message, Name};
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::authz::v1beta1::{self as authz, *},
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The authz module's query client proto definition
pub type AuthzQueryClient = authz::query_client::QueryClient<Channel>;
#[allow(missing_docs)]
pub type Grant = authz::Grant;

#[async_trait]
impl ConstructClient<AuthzQueryClient> for AuthzQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        AuthzQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets all grants between `granter` and `grantee` for the given msg type
    pub async fn query_grants(
        &mut self,
        granter: &str,
        grantee: &str,
        msg_type_url: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryGrantsResponse> {
        let query_client = self.get_client::<AuthzQueryClient>().await?;
        let request = QueryGrantsRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: msg_type_url.to_string(),
            // TODO: Support pagination if use case arises
            pagination,
        };

        Ok(query_client.grants(request).await?.into_inner())
    }

    /// Gets all grant authorizations granted by the provided `granter`
    pub async fn query_granter_grants(
        &mut self,
        granter: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryGranterGrantsResponse> {
        let query_client = self.get_client::<AuthzQueryClient>().await?;
        let request = QueryGranterGrantsRequest {
            granter: granter.to_string(),
            pagination,
        };

        Ok(query_client.granter_grants(request).await?.into_inner())
    }

    /// Gets all grant authorizations granted to the provided `grantee`
    pub async fn query_grantee_grants(
        &mut self,
        grantee: &str,
        pagination: Option<PageRequest>,
    ) -> Result<QueryGranteeGrantsResponse> {
        let query_client = self.get_client::<AuthzQueryClient>().await?;
        let request = QueryGranteeGrantsRequest {
            grantee: grantee.to_string(),
            pagination,
        };

        Ok(query_client.grantee_grants(request).await?.into_inner())
    }
}

/// Represents a [Authz module message](https://docs.cosmos.network/v0.45/modules/authz/03_messages.html)
#[derive(Clone, Debug)]
pub enum Authz<'m> {
    /// Authorize one account to execute a specified message on behalf of another. Represents a [`MsgGrant`]
    Grant {
        /// Address of the granter account
        granter: &'m str,
        /// Address of the grantee account
        grantee: &'m str,
        /// Authorization to be granted to grantee
        grant: Grant,
    },
    /// Revoke authorization of a previously created [`Grant`]. Represents a [`MsgRevoke`]
    Revoke {
        /// Address of the granter account
        granter: &'m str,
        /// Address of the grantee account
        grantee: &'m str,
        /// Type URL of the message previously authorized for grantee
        msg_type_url: &'m str,
    },
    /// Execute a message on behalf of another account under the authorization of a previously created [`Grant`]. Represents a [`MsgExec`]
    Exec {
        /// Address of the grantee account
        grantee: &'m str,
        /// Messages to be executed
        msgs: Vec<Any>,
    },
}

impl ModuleMsg for Authz<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Authz::Grant {
                granter,
                grantee,
                grant,
            } => MsgGrant {
                granter: AccountId::from_str(granter)?,
                grantee: AccountId::from_str(grantee)?,
                grant,
            }
            .to_any(),
            Authz::Revoke {
                granter,
                grantee,
                msg_type_url,
            } => MsgRevoke {
                granter: AccountId::from_str(granter)?,
                grantee: AccountId::from_str(grantee)?,
                msg_type_url: msg_type_url.to_string(),
            }
            .to_any(),
            Authz::Exec { grantee, msgs } => MsgExec {
                grantee: AccountId::from_str(grantee)?,
                msgs,
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

/// Implemention of [`cosmrs::tx::Msg`] for `MsgGrant`
#[derive(Debug, Default)]
pub struct WrappedMsgGrant {
    inner: cosmrs::proto::cosmos::authz::v1beta1::MsgGrant,
}

impl Name for WrappedMsgGrant {
    const NAME: &'static str = "MsgGrant";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
}

impl Message for WrappedMsgGrant {
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

/// Represents a message to grant authorization to execute a Msg from `granter` to `grantee`
#[derive(Clone, Debug)]
pub struct MsgGrant {
    /// Granting account's address.
    pub granter: AccountId,

    /// Grantee's address.
    pub grantee: AccountId,

    /// The scope of authority granted to the `grantee`, including authorized messages and expiry.
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
            grant: proto
                .inner
                .grant
                .clone()
                .ok_or(eyre!("grant cannot be empty"))?,
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
                // TO-DO: Grant type needs a wrapper, it's cumbersome
                grant: Some(msg.grant.clone()),
            },
        }
    }
}

/// Implemention of [`cosmrs::tx::Msg`] for `MsgRevoke`
#[derive(Debug, Default)]
pub struct WrappedMsgRevoke {
    inner: cosmrs::proto::cosmos::authz::v1beta1::MsgRevoke,
}

impl Name for WrappedMsgRevoke {
    const NAME: &'static str = "MsgRevoke";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
}

impl Message for WrappedMsgRevoke {
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

/// MsgRevoke represents a message to revoke a [`Grant`] from `granter` to `grantee`
#[derive(Clone, Debug)]
pub struct MsgRevoke {
    /// Granter's address.
    pub granter: AccountId,

    /// Grantee's address.
    pub grantee: AccountId,

    /// Type URL of the message to revoke authorization for
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
                msg_type_url: msg.msg_type_url.clone(),
            },
        }
    }
}

/// Implemention of [`cosmrs::tx::Msg`] for `MsgExec`
#[derive(Debug, Default)]
pub struct WrappedMsgExec {
    inner: cosmrs::proto::cosmos::authz::v1beta1::MsgExec,
}

impl Name for WrappedMsgExec {
    const NAME: &'static str = "MsgExec";
    const PACKAGE: &'static str = "cosmos.authz.v1beta1";
}

impl Message for WrappedMsgExec {
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
                msgs: msg.msgs.clone(),
            },
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
            },
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
            msgs: vec![],
        }
        .into_tx()
        .unwrap();
    }
}
