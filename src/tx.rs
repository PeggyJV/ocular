//! Defines core types for building and executing module Msgs and transactions.
//!
//! ## Examples
//!
//! One-off send transaction
//!
//! ```ignore
//! // arguments construction not shown
//! let response = Bank::Send {
//!     from: "cosmos1y6d5kasehecexf09ka6y0ggl0pxzt6dgk0gnl9",
//!     to: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
//!     amount: 100000,
//!     denom: "uatom"
//! }
//! .into_tx()?
//! .sign(
//!     &signer,
//!     fee_info,
//!     chain_context,
//!     &mut qclient,
//! )
//! .broadcast_commit(&mut mclient)
//! .await?;
//! ```
//!
//! Building a tx with multiple Msgs
//!
//! ```ignore
//! // msg construction not shown
//! let mut unsigned_tx = UnsignedTx::new();
//!
//! unsigned_tx.add_msg(msg1);
//! unsigned_tx.add_msg(msg2);
//! unsigned_tx.memo("Arbitrage!");
//!
//! // ...
//! ```

use cosmrs::AccountId;
use eyre::{eyre, Result};

use crate::{
    account::AccountInfo,
    chain::ChainContext,
    cosmrs::{
        tx::{BodyBuilder, Fee, Raw, SignDoc, SignerInfo},
        Any, Coin,
    },
    grpc::GrpcClient,
};

/// Convenience wrapper around a [`BodyBuilder`] representing an unsigned tx
#[derive(Clone, Debug)]
pub struct UnsignedTx {
    inner: BodyBuilder,
}

impl UnsignedTx {
    /// Constructs an empty [`UnsignedTx`]
    pub fn new() -> Self {
        UnsignedTx {
            inner: BodyBuilder::new(),
        }
    }

    /// Adds a Msg to the transaction
    pub fn add_msg(&mut self, msg: impl Into<Any>) {
        self.inner.msg(msg);
    }

    /// Adds multiple Msgs to the transaction
    pub fn add_msgs(&mut self, msgs: impl IntoIterator<Item = Any>) {
        self.inner.msgs(msgs);
    }

    /// Adds an extension option
    pub fn add_extension_option(&mut self, value: impl Into<Any>) {
        self.inner.extension_option(value);
    }

    /// Adds a non-critical extension option
    pub fn add_non_critical_extension_option(&mut self, value: impl Into<Any>) {
        self.inner.non_critical_extension_option(value);
    }

    /// Adds a memo
    pub fn memo(&mut self, value: impl Into<String>) {
        self.inner.memo(value);
    }

    /// Adds a timeout height
    pub fn timeout_height(&mut self, value: u32) {
        self.inner.timeout_height(value);
    }

    /// Converts to the inner [`BodyBuilder`]
    pub fn into_inner(self) -> BodyBuilder {
        self.inner
    }

    /// Signs the inner tx body by first querying for the account's number and sequence.
    pub async fn sign(
        self,
        signer: &AccountInfo,
        fee_info: FeeInfo,
        chain_context: &ChainContext,
        client: &mut GrpcClient,
    ) -> Result<SignedTx> {
        let address = signer.address(&chain_context.prefix)?;
        let account = client.query_account(&address).await?;

        self.sign_with_sequence(
            signer,
            fee_info,
            chain_context,
            account.account_number,
            account.sequence,
        )
    }

    /// Signs the inner tx body with the provided account number and sequence. Useful in cases where optimizing tx
    /// speed is important.
    pub fn sign_with_sequence(
        self,
        signer: &AccountInfo,
        fee_info: FeeInfo,
        chain_context: &ChainContext,
        account_number: u64,
        sequence: u64,
    ) -> Result<SignedTx> {
        let signer_info = SignerInfo::single_direct(Some(signer.public_key()), sequence);
        let auth_info = signer_info.auth_info(Fee {
            amount: vec![fee_info.fee],
            gas_limit: fee_info.gas_limit.into(),
            payer: fee_info.fee_payer,
            granter: fee_info.fee_granter,
        });
        let chain_id = &cosmrs::tendermint::chain::Id::try_from(chain_context.id.to_owned())?;
        let tx_body = self.into_inner().finish();
        let sign_doc = match SignDoc::new(&tx_body, &auth_info, chain_id, account_number) {
            Ok(doc) => doc,
            Err(err) => {
                return Err(eyre!(
                    "Failed to create SignDoc for chain {}: {}",
                    chain_id,
                    err
                ))
            }
        };
        let tx_signed = match sign_doc.sign(signer.private_key()) {
            Ok(raw) => raw,
            Err(err) => return Err(eyre!("Failed to sign tx for chain {}: {}", chain_id, err)),
        };

        Ok(SignedTx { inner: tx_signed })
    }
}

impl Default for UnsignedTx {
    fn default() -> Self {
        Self::new()
    }
}

impl From<BodyBuilder> for UnsignedTx {
    fn from(builder: BodyBuilder) -> Self {
        UnsignedTx { inner: builder }
    }
}

impl From<&BodyBuilder> for UnsignedTx {
    fn from(builder: &BodyBuilder) -> Self {
        UnsignedTx {
            inner: builder.clone(),
        }
    }
}

impl From<&mut BodyBuilder> for UnsignedTx {
    fn from(builder: &mut BodyBuilder) -> Self {
        UnsignedTx {
            inner: builder.clone(),
        }
    }
}

impl From<Any> for UnsignedTx {
    fn from(msg: Any) -> Self {
        let mut builder = BodyBuilder::new();
        builder.msg(msg);
        builder.into()
    }
}

impl From<&Any> for UnsignedTx {
    fn from(msg: &Any) -> Self {
        let mut builder = BodyBuilder::new();
        builder.msg(msg.to_owned());
        builder.into()
    }
}

impl From<&Vec<Any>> for UnsignedTx {
    fn from(msgs: &Vec<Any>) -> Self {
        let mut builder = BodyBuilder::new();
        builder.msgs(msgs.to_owned());
        builder.into()
    }
}

/// Wrapper around a [`Raw`], the raw bytes of a signed tx
#[derive(Debug)]
pub struct SignedTx {
    inner: Raw,
}

impl SignedTx {
    /// Converts to the inner [`Raw`]
    pub fn into_inner(self) -> Raw {
        self.inner
    }

    /// Converts to the bytes of the signed transaction
    pub fn to_bytes(self) -> Result<Vec<u8>> {
        self.inner.to_bytes()
    }
}

/// Wrapper for fee/gas related tx configuration
#[derive(Clone, Debug)]
pub struct FeeInfo {
    fee: Coin,
    fee_payer: Option<AccountId>,
    fee_granter: Option<AccountId>,
    gas_limit: u64,
}

impl FeeInfo {
    /// Constructor
    pub fn new(fee: Coin) -> Self {
        FeeInfo {
            fee,
            fee_payer: None,
            fee_granter: None,
            gas_limit: 200_000,
        }
    }

    /// Gets a reference to the inner fee
    pub fn get_fee(&self) -> &Coin {
        &self.fee
    }

    /// Set the inner fee's amount field
    pub fn amount(&mut self, value: u128) {
        self.fee.amount = value;
    }

    /// Set the fee payer. If [`None`] (default), the first signer is responsible for paying the fees.
    ///
    /// If [`Some`], the specified account must pay the fees. The payer must be
    /// a tx signer.
    ///
    /// Setting this field does not change the ordering of required signers for
    /// the transaction.
    pub fn fee_payer(&mut self, value: Option<AccountId>) {
        self.fee_payer = value;
    }

    /// Set the fee granter. If [`Some`], the fee payer (either the first signer or the
    /// value of the payer field) requests that a fee grant be used to pay fees
    /// instead of the fee payer’s own balance.
    ///
    /// If an appropriate fee grant does not exist or the chain does not
    /// support fee grants, this will fail.
    pub fn fee_granter(&mut self, value: Option<AccountId>) {
        self.fee_granter = value;
    }

    /// Set the gas limit (GasWanted) of the transaction
    pub fn gas_limit(&mut self, value: u64) {
        self.gas_limit = value;
    }
}

/// Represents an arbitrary Cosmos module Msg
pub trait ModuleMsg
where
    Self::Error: core::fmt::Debug,
{
    #[allow(missing_docs)]
    type Error;

    /// Attempts to convert the message into an [`Any`] for inclusion in a transaction
    fn into_any(self) -> Result<Any, Self::Error>;
    /// Attempts to construct an [`UnsignedTx`] containing the message.
    fn into_tx(self) -> Result<UnsignedTx, Self::Error>;
}
