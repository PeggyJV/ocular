//! Queries and messages for the [Slashing module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/slashing/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`SlashingQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{tx::Msg, AccountId, Any};
use eyre::{Context, Report, Result};
use prost::{Message, Name};
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::slashing::v1beta1 as slashing,
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The slashing module's query client proto definition
pub type SlashingQueryClient = slashing::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<SlashingQueryClient> for SlashingQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        SlashingQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Params queries the parameters of slashing module
    pub async fn query_slashing_params(&mut self) -> Result<slashing::QueryParamsResponse> {
        let query_client = self.get_client::<SlashingQueryClient>().await?;
        let request = slashing::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }

    /// SigningInfo queries the signing info of given cons address
    pub async fn query_signing_info(
        &mut self,
        cons_address: &str,
    ) -> Result<slashing::QuerySigningInfoResponse> {
        let query_client = self.get_client::<SlashingQueryClient>().await?;
        let request = slashing::QuerySigningInfoRequest {
            cons_address: cons_address.to_string(),
        };

        Ok(query_client.signing_info(request).await?.into_inner())
    }

    /// SigningInfos queries signing info of all validators
    pub async fn query_signing_infos(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<slashing::QuerySigningInfosResponse> {
        let query_client = self.get_client::<SlashingQueryClient>().await?;
        let request = slashing::QuerySigningInfosRequest { pagination };

        Ok(query_client.signing_infos(request).await?.into_inner())
    }
}

/// Represents a [Slashing module message](https://docs.cosmos.network/v0.45/modules/slashing/03_messages.html)
#[derive(Clone, Debug)]
pub enum Slashing<'m> {
    /// Unjail a jailed validator. Represents a [`MsgUnjail`]
    Unjail {
        /// Address of the validator to unjail
        validator_address: &'m str,
    },
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

/// We implement cosmrs::tx::Msg for slashing Msgs because they're not in cosmrs
#[derive(Debug, Default)]
pub struct WrappedMsgVerifyInvariant {
    inner: cosmrs::proto::cosmos::slashing::v1beta1::MsgUnjail,
}

impl Name for WrappedMsgVerifyInvariant {
    const NAME: &'static str = "MsgVerifyInvariant";
    const PACKAGE: &'static str = "cosmos.slashing.v1beta1";
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
