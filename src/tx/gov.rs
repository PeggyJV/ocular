#![allow(dead_code)]
//! Messages for participating in governance
//!
//! Since cosmrs doesn't currently have [`cosmrs::tx::msg::Msg`] implementations for Gov messages,
//! they are defined here.
use std::str::FromStr;

use eyre::{eyre, Report, Result};
use cosmrs::{proto::{cosmos::gov::v1beta1::VoteOption, traits::TypeUrl}, Any, AccountId, tx::Msg, Coin, Denom};
use prost::Message;

use super::{ModuleMsg, UnsignedTx};

/// Represents a [Gov module message](https://docs.cosmos.network/v0.45/modules/gov/03_messages.html)
#[derive(Clone, Debug)]
pub enum Gov<'m> {
    /// Submit a proposal to governance. Represents a [`MsgSubmitProposal`]
    SubmitProposal {
        content: Any,
        amount: u128,
        denom: &'m str,
        proposer: &'m str,
    },
    /// Make a deposit to fund a proposal. Represents a [`MsgDeposit`]
    Deposit {
        proposal_id: u64,
        depositor: &'m str,
        amount: u128,
        denom: &'m str,
    },
    /// Vote on a proposal. Represents a [`MsgVote`]
    Vote {
        proposal_id: u64,
        voter: &'m str,
        option: VoteOption,
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
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };

                MsgSubmitProposal {
                    content,
                    initial_deposit,
                    proposer: AccountId::from_str(proposer)?,
                }
                .to_any()
            },
            Gov::Deposit {
                proposal_id,
                depositor,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };

                MsgDeposit {
                    proposal_id,
                    depositor: AccountId::from_str(depositor)?,
                    amount,
                }
                .to_any()
            },
            Gov::Vote {
                proposal_id,
                voter,
                option,
            } => {
                MsgVote {
                    proposal_id,
                    voter: AccountId::from_str(voter)?,
                    option,
                }
                .to_any()
            }
        }
    }

    /// Converts the message enum representation into an [`UnsignedTx`] containing the corresponding Msg
    fn into_tx(self) -> Result<UnsignedTx> {
        let mut tx = UnsignedTx::new();
        tx.add_msg(self.into_any()?);

        Ok(tx)
    }
}

// We implement cosmrs::tx::Msg for gov Msgs because they're not in cosmrs
#[derive(Debug)]
pub struct WrappedMsgSubmitProposal {
    inner: cosmrs::proto::cosmos::gov::v1beta1::MsgSubmitProposal,
}

impl Message for WrappedMsgSubmitProposal {
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

impl Default for WrappedMsgSubmitProposal {
    fn default() -> Self {
        WrappedMsgSubmitProposal {
            inner: cosmrs::proto::cosmos::gov::v1beta1::MsgSubmitProposal::default()
        }
    }
}

impl TypeUrl for WrappedMsgSubmitProposal {
    const TYPE_URL: &'static str = "/cosmos.gov.v1beta1.MsgSubmitProposal";
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
            return Err(eyre!("initial deposit cannot be empty"))
        }

        Ok(MsgSubmitProposal {
            content: proto.inner.content.clone().ok_or(eyre!("content cannot be empty"))?,
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
            }
        }
    }
}

#[derive(Debug)]
pub struct WrappedMsgDeposit {
    inner: cosmrs::proto::cosmos::gov::v1beta1::MsgDeposit,
}

impl Message for WrappedMsgDeposit {
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

impl Default for WrappedMsgDeposit {
    fn default() -> Self {
        WrappedMsgDeposit {
            inner: cosmrs::proto::cosmos::gov::v1beta1::MsgDeposit::default()
        }
    }
}

impl TypeUrl for WrappedMsgDeposit {
    const TYPE_URL: &'static str = "/cosmos.gov.v1beta1.MsgDeposit";
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
                amount: vec![msg.amount.clone().into()]
            }
        }
    }
}

#[derive(Debug)]
pub struct WrappedMsgVote {
    inner: cosmrs::proto::cosmos::gov::v1beta1::MsgVote,
}

impl Message for WrappedMsgVote {
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

impl Default for WrappedMsgVote {
    fn default() -> Self {
        WrappedMsgVote {
            inner: cosmrs::proto::cosmos::gov::v1beta1::MsgVote::default()
        }
    }
}

impl TypeUrl for WrappedMsgVote {
    const TYPE_URL: &'static str = "/cosmos.gov.v1beta1.MsgVote";
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
                _ => return Err(eyre!("invalid vote option index"))
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            denom: "uatom"
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
