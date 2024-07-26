//! Queries and messages for the [Gov module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/gov/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`GovQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{proto::cosmos::gov::v1beta1::VoteOption, tx::Msg, AccountId, Any, Coin, Denom};
use eyre::{eyre, Context, Report, Result};
use prost::{Message, Name};
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::gov::v1beta1::{
        self as gov, QueryProposalsResponse, QueryVotesResponse,
    },
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The gov module's query client proto definition
pub type GovQueryClient = gov::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<GovQueryClient> for GovQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        GovQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Params queries all parameters of the gov module.
    pub async fn query_gov_params(
        &mut self,
        params_type: String,
    ) -> Result<gov::QueryParamsResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryParamsRequest { params_type };

        Ok(query_client.params(request).await?.into_inner())
    }

    /// Proposal queries proposal details based on ProposalID.
    pub async fn query_proposal(&mut self, proposal_id: u64) -> Result<gov::QueryProposalResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryProposalRequest { proposal_id };

        Ok(query_client.proposal(request).await?.into_inner())
    }

    /// Proposals queries all proposals based on given status.
    pub async fn query_proposals(
        &mut self,
        proposal_status: i32,
        voter: String,
        depositor: String,
        pagination: Option<PageRequest>,
    ) -> Result<QueryProposalsResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryProposalsRequest {
            proposal_status,
            voter,
            depositor,
            pagination,
        };

        Ok(query_client.proposals(request).await?.into_inner())
    }

    /// Vote queries voted information based on proposalID, voterAddr.
    pub async fn query_vote(
        &mut self,
        proposal_id: u64,
        voter: String,
    ) -> Result<gov::QueryVoteResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryVoteRequest { proposal_id, voter };

        Ok(query_client.vote(request).await?.into_inner())
    }

    /// Votes queries votes of a given proposal.
    pub async fn query_votes(
        &mut self,
        proposal_id: u64,
        pagination: Option<PageRequest>,
    ) -> Result<QueryVotesResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryVotesRequest {
            proposal_id,
            pagination,
        };

        Ok(query_client.votes(request).await?.into_inner())
    }

    /// Deposit queries single deposit information based proposalID, depositAddr.
    pub async fn query_deposit(
        &mut self,
        proposal_id: u64,
        depositor: String,
    ) -> Result<gov::QueryDepositResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryDepositRequest {
            proposal_id,
            depositor,
        };

        Ok(query_client.deposit(request).await?.into_inner())
    }

    /// Deposits queries all deposits of a single proposal.
    pub async fn query_deposits(
        &mut self,
        proposal_id: u64,
        pagination: Option<PageRequest>,
    ) -> Result<gov::QueryDepositsResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryDepositsRequest {
            proposal_id,
            pagination,
        };

        Ok(query_client.deposits(request).await?.into_inner())
    }

    /// TallyResult queries the tally of a proposal vote.
    pub async fn query_tally_result(
        &mut self,
        proposal_id: u64,
    ) -> Result<gov::QueryTallyResultResponse> {
        let query_client = self.get_client::<GovQueryClient>().await?;
        let request = gov::QueryTallyResultRequest { proposal_id };

        Ok(query_client.tally_result(request).await?.into_inner())
    }
}

/// Represents a [Gov module message](https://docs.cosmos.network/v0.45/modules/gov/03_messages.html)
#[derive(Clone, Debug)]
pub enum Gov<'m> {
    /// Submit a proposal to governance. Represents a [`MsgSubmitProposal`]
    SubmitProposal {
        /// Description content of the proposal
        content: Any,
        /// Initial deposit amount
        amount: u128,
        /// Initial deposit coin denomation
        denom: &'m str,
        /// Proposer's account address
        proposer: &'m str,
    },
    /// Make a deposit to fund a proposal. Represents a [`MsgDeposit`]
    Deposit {
        /// ID of the proposal
        proposal_id: u64,
        /// Address of the depositing account
        depositor: &'m str,
        /// Deposit amount
        amount: u128,
        /// Deposit coin denomination
        denom: &'m str,
    },
    /// Vote on a proposal. Represents a [`MsgVote`]
    Vote {
        /// ID of the proposal
        proposal_id: u64,
        /// Address of the voting account
        voter: &'m str,
        /// Vote option
        option: gov::VoteOption,
    },
}

impl ModuleMsg for Gov<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Gov::SubmitProposal {
                content,
                amount,
                denom,
                proposer,
            } => {
                let initial_deposit = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };

                MsgSubmitProposal {
                    content,
                    initial_deposit,
                    proposer: AccountId::from_str(proposer)?,
                }
                .to_any()
            }
            Gov::Deposit {
                proposal_id,
                depositor,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };

                MsgDeposit {
                    proposal_id,
                    depositor: AccountId::from_str(depositor)?,
                    amount,
                }
                .to_any()
            }
            Gov::Vote {
                proposal_id,
                voter,
                option,
            } => MsgVote {
                proposal_id,
                voter: AccountId::from_str(voter)?,
                option,
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

/// Implementation of [`cosmrs::tx::Msg`] for `MsgSubmitProposal`
#[derive(Debug, Default)]
pub struct WrappedMsgSubmitProposal {
    inner: cosmrs::proto::cosmos::gov::v1beta1::MsgSubmitProposal,
}

impl Name for WrappedMsgSubmitProposal {
    const NAME: &'static str = "MsgSubmitProposal";
    const PACKAGE: &'static str = "cosmos.gov.v1beta1";
}

impl Message for WrappedMsgSubmitProposal {
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

/// MsgSubmitProposal represents a message to submit a governance proposal
#[derive(Clone, Debug)]
pub struct MsgSubmitProposal {
    /// Content of the proposal
    pub content: Any,

    /// Initial deposit required to submit the proposal
    pub initial_deposit: Coin,

    /// Proposor's address.
    pub proposer: AccountId,
}

impl Msg for MsgSubmitProposal {
    type Proto = WrappedMsgSubmitProposal;
}

impl TryFrom<WrappedMsgSubmitProposal> for MsgSubmitProposal {
    type Error = Report;

    fn try_from(proto: WrappedMsgSubmitProposal) -> Result<MsgSubmitProposal> {
        MsgSubmitProposal::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgSubmitProposal> for MsgSubmitProposal {
    type Error = Report;

    fn try_from(proto: &WrappedMsgSubmitProposal) -> Result<MsgSubmitProposal> {
        if proto.inner.initial_deposit.is_empty() {
            return Err(eyre!("initial deposit cannot be empty"));
        }

        Ok(MsgSubmitProposal {
            content: proto
                .inner
                .content
                .clone()
                .ok_or(eyre!("content cannot be empty"))?,
            initial_deposit: Coin::try_from(proto.inner.initial_deposit[0].clone())?,
            proposer: AccountId::from_str(&proto.inner.proposer)?,
        })
    }
}

impl From<MsgSubmitProposal> for WrappedMsgSubmitProposal {
    fn from(coin: MsgSubmitProposal) -> WrappedMsgSubmitProposal {
        WrappedMsgSubmitProposal::from(&coin)
    }
}

impl From<&MsgSubmitProposal> for WrappedMsgSubmitProposal {
    fn from(msg: &MsgSubmitProposal) -> WrappedMsgSubmitProposal {
        WrappedMsgSubmitProposal {
            inner: cosmrs::proto::cosmos::gov::v1beta1::MsgSubmitProposal {
                content: Some(msg.content.to_owned()),
                initial_deposit: vec![msg.initial_deposit.to_owned().into()],
                proposer: msg.proposer.to_string(),
            },
        }
    }
}

/// Implementation of [`cosmrs::tx::Msg`] for `MsgDeposit`
#[derive(Debug, Default)]
pub struct WrappedMsgDeposit {
    inner: cosmrs::proto::cosmos::gov::v1beta1::MsgDeposit,
}

impl Name for WrappedMsgDeposit {
    const NAME: &'static str = "MsgDeposit";
    const PACKAGE: &'static str = "cosmos.gov.v1beta1";
}

impl Message for WrappedMsgDeposit {
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

/// MsgDeposit represents a message to fund a proposal
#[derive(Clone, Debug)]
pub struct MsgDeposit {
    /// ID of the proposal
    proposal_id: u64,

    /// Depositor's address
    depositor: AccountId,

    /// Deposit amount
    amount: Coin,
}

impl Msg for MsgDeposit {
    type Proto = WrappedMsgDeposit;
}

impl TryFrom<WrappedMsgDeposit> for MsgDeposit {
    type Error = Report;

    fn try_from(proto: WrappedMsgDeposit) -> Result<MsgDeposit> {
        MsgDeposit::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgDeposit> for MsgDeposit {
    type Error = Report;

    fn try_from(proto: &WrappedMsgDeposit) -> Result<MsgDeposit> {
        Ok(MsgDeposit {
            proposal_id: proto.inner.proposal_id,
            depositor: AccountId::from_str(&proto.inner.depositor)?,
            amount: Coin::try_from(proto.inner.amount[0].to_owned())?,
        })
    }
}

impl From<MsgDeposit> for WrappedMsgDeposit {
    fn from(coin: MsgDeposit) -> WrappedMsgDeposit {
        WrappedMsgDeposit::from(&coin)
    }
}

impl From<&MsgDeposit> for WrappedMsgDeposit {
    fn from(msg: &MsgDeposit) -> WrappedMsgDeposit {
        WrappedMsgDeposit {
            inner: cosmrs::proto::cosmos::gov::v1beta1::MsgDeposit {
                proposal_id: msg.proposal_id,
                depositor: msg.depositor.to_string(),
                amount: vec![msg.amount.clone().into()],
            },
        }
    }
}

/// Implementation of [`cosmrs::tx::Msg`] for `MsgVote`
#[derive(Debug, Default)]
pub struct WrappedMsgVote {
    inner: cosmrs::proto::cosmos::gov::v1beta1::MsgVote,
}

impl Name for WrappedMsgVote {
    const NAME: &'static str = "MsgVote";
    const PACKAGE: &'static str = "cosmos.gov.v1beta1";
}

impl Message for WrappedMsgVote {
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

/// MsgVote represents a message to vote on a proposal
#[derive(Clone, Debug)]
pub struct MsgVote {
    /// ID of the proposal
    pub proposal_id: u64,

    /// Voter's address.
    pub voter: AccountId,

    /// Vote option
    pub option: VoteOption,
}

impl Msg for MsgVote {
    type Proto = WrappedMsgVote;
}

impl TryFrom<WrappedMsgVote> for MsgVote {
    type Error = Report;

    fn try_from(proto: WrappedMsgVote) -> Result<MsgVote> {
        MsgVote::try_from(&proto)
    }
}

impl TryFrom<&WrappedMsgVote> for MsgVote {
    type Error = Report;

    fn try_from(proto: &WrappedMsgVote) -> Result<MsgVote> {
        Ok(MsgVote {
            proposal_id: proto.inner.proposal_id,
            voter: AccountId::from_str(&proto.inner.voter)?,
            option: match proto.inner.option {
                0 => VoteOption::Unspecified,
                1 => VoteOption::Yes,
                2 => VoteOption::Abstain,
                3 => VoteOption::No,
                4 => VoteOption::NoWithVeto,
                _ => return Err(eyre!("invalid vote option index")),
            },
        })
    }
}

impl From<MsgVote> for WrappedMsgVote {
    fn from(coin: MsgVote) -> WrappedMsgVote {
        WrappedMsgVote::from(&coin)
    }
}

impl From<&MsgVote> for WrappedMsgVote {
    fn from(msg: &MsgVote) -> WrappedMsgVote {
        WrappedMsgVote {
            inner: cosmrs::proto::cosmos::gov::v1beta1::MsgVote {
                proposal_id: msg.proposal_id,
                voter: msg.voter.to_string(),
                option: msg.option.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmrs::{proto::cosmos::gov::v1beta1::VoteOption, Any};

    use super::*;

    #[test]
    fn construct_txs() {
        Gov::SubmitProposal {
            proposer: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            content: Any::default(),
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Gov::Deposit {
            proposal_id: 0,
            depositor: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Gov::Vote {
            proposal_id: 0,
            voter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            option: VoteOption::Yes,
        }
        .into_tx()
        .unwrap();
    }
}
