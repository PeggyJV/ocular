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
use tendermint_rpc::{
    endpoint::broadcast::{tx_async, tx_commit, tx_sync},
    Client,
};

use crate::{
    account::AccountInfo,
    chain::Context,
    cosmrs::{
        tx::{BodyBuilder, Fee, Raw, SignDoc, SignerInfo},
        Any, Coin,
    },
    rpc::{new_http_client, RpcHttpClient},
    QueryClient,
};

pub mod authz;
pub mod bank;
pub mod crisis;
pub mod distribution;
pub mod evidence;
pub mod feegrant;
pub mod gov;
pub mod slashing;
pub mod staking;

/// Client for broadcasting [`SignedTx`]
pub struct MsgClient {
    inner: RpcHttpClient,
}

impl MsgClient {
    /// Constructor
    pub fn new(rpc_endpoint: &str) -> Result<MsgClient> {
        let inner = new_http_client(rpc_endpoint)?;

        Ok(MsgClient { inner })
    }

    /// Gets a reference to the the inner RPC client
    pub fn inner(&self) -> &RpcHttpClient {
        &self.inner
    }
}

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
        chain_context: Context,
        qclient: &mut QueryClient,
    ) -> Result<SignedTx> {
        let address = signer.address(&chain_context.prefix)?;
        let account = qclient.account(&address).await?;

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
        chain_context: Context,
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
        let chain_id = &cosmrs::tendermint::chain::Id::try_from(chain_context.id)?;
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

/// Wrapper around a [`Raw`], the raw bytes of a signed tx
#[derive(Debug)]
pub struct SignedTx {
    inner: Raw,
}

impl SignedTx {
    /// Broadcasts transaction using the /broadcast_async Tendermint endpoint. Returns right away without waiting on CheckTx.
    pub async fn broadcast_async(self, client: &mut MsgClient) -> Result<tx_async::Response> {
        let tx = self.to_bytes()?.into();
        client
            .inner()
            .broadcast_tx_async(tx)
            .await
            .map_err(|e| e.into())
    }

    /// Broadcasts transaction using the /broadcast_commit Tendermint endpoint, waiting for CheckTx and DeliverTx to complete
    /// before returning. Note that the server may time out the connection while waiting for the tx to be included in a block.
    /// This can result in an error being returned by this method even if the tx is ultimately successful.
    pub async fn broadcast_commit(self, client: &mut MsgClient) -> Result<tx_commit::Response> {
        self.inner.broadcast_commit(client.inner()).await
    }

    /// Broadcasts transaction using the /broadcast_sync Tendermint endpoint. Waits for CheckTx but not DeliverTx.
    pub async fn broadcast_sync(self, client: &mut MsgClient) -> Result<tx_sync::Response> {
        let tx = self.to_bytes()?.into();
        client
            .inner()
            .broadcast_tx_sync(tx)
            .await
            .map_err(|e| e.into())
    }

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
    pub fn new(fee: Coin) -> Self {
        FeeInfo {
            fee,
            fee_payer: None,
            fee_granter: None,
            gas_limit: 200_000,
        }
    }

    pub fn amount(&mut self, value: u128) {
        self.fee.amount = value;
    }

    pub fn fee_payer(&mut self, value: Option<AccountId>) {
        self.fee_payer = value;
    }

    pub fn fee_granter(&mut self, value: Option<AccountId>) {
        self.fee_granter = value;
    }

    pub fn gas_limit(&mut self, value: u64) {
        self.gas_limit = value;
    }
}

/// Represents an arbitrary Cosmos module Msg
pub trait ModuleMsg {
    type Error;

    fn into_any(self) -> Result<Any, Self::Error>;
    fn into_tx(self) -> Result<UnsignedTx, Self::Error>;
}
