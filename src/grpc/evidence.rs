//! Queries and messages for the [Evidence module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/evidence/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`EvidenceQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{
    proto::{cosmos::evidence::v1beta1::QueryAllEvidenceResponse, traits::TypeUrl},
    tx::Msg,
    AccountId, Any,
};
use eyre::{eyre, Context, Report, Result};
use prost::Message;
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::evidence::v1beta1 as evidence,
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The evidence module's query client proto definition
pub type EvidenceQueryClient = evidence::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<EvidenceQueryClient> for EvidenceQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        EvidenceQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets evidence with the specified hash. Hash must be a valid hex string.
    pub async fn query_evidence(
        &mut self,
        evidence_hash: String,
    ) -> Result<evidence::QueryEvidenceResponse> {
        let query_client = self.get_client::<EvidenceQueryClient>().await?;
        let request = evidence::QueryEvidenceRequest {
            evidence_hash: hex::decode(evidence_hash)?,
        };

        Ok(query_client.evidence(request).await?.into_inner())
    }

    /// Gets all evidence with optional pagination
    pub async fn query_all_evidence(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<QueryAllEvidenceResponse> {
        let query_client = self.get_client::<EvidenceQueryClient>().await?;
        let request = evidence::QueryAllEvidenceRequest { pagination };

        Ok(query_client.all_evidence(request).await?.into_inner())
    }
}

/// Represents a [Evidence module message](https://docs.cosmos.network/v0.45/modules/evidence/03_messages.html)
#[derive(Clone, Debug)]
pub enum Evidence<'m> {
    /// Submit evidence of malicious behavior by a validator for slashing. To learn more, see
    /// [Evidence](https://docs.cosmos.network/master/modules/evidence/). Represents a [`MsgSubmitEvidence`]
    SubmitEvidence {
        /// Address of the account submitting evidence
        submitter: &'m str,
        /// Evidence to be submitted
        evidence: Any,
    },
}

impl ModuleMsg for Evidence<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Evidence::SubmitEvidence {
                submitter,
                evidence,
            } => MsgSubmitEvidence {
                submitter: AccountId::from_str(submitter)?,
                evidence,
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

/// We implement cosmrs::tx::Msg for evidence Msgs because they're not in cosmrs
#[derive(Debug, Default)]
pub struct WrappedMsgSubmitEvidence {
    inner: cosmrs::proto::cosmos::evidence::v1beta1::MsgSubmitEvidence,
}

impl Message for WrappedMsgSubmitEvidence {
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

impl TypeUrl for WrappedMsgSubmitEvidence {
    const TYPE_URL: &'static str = "/cosmos.evidence.v1beta1.MsgSubmitEvidence";
}

/// MsgSubmitEvidence represents a message to send evidence of malicious conduct by a validator for slashing
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
            evidence: proto
                .inner
                .evidence
                .to_owned()
                .ok_or(eyre!("evidence cannot be empty"))?,
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
                evidence: Some(msg.evidence.to_owned()),
            },
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
