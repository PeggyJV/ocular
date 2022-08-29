#![cfg(feature = "tx")]
//! Defines core types for building and executing module Msgs and transactions.
use cosmrs::AccountId;
use eyre::{eyre, Result};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use crate::{
    account::AccountInfo,
    chain::Context,
    cosmrs::{
        tx::{BodyBuilder, Fee, Raw, SignDoc, SignerInfo},
        Any, Coin,
    },
    QueryClient,
    rpc::{RpcHttpClient, new_http_client},
};

mod bank;

/// Client for broadcasting [`SignedTx`]
pub struct MsgClient {
    inner: RpcHttpClient,
}

impl MsgClient {
    /// Constructor
    pub fn new(rpc_endpoint: &str) -> Result<MsgClient> {
        let inner = new_http_client(rpc_endpoint)?;

        Ok(MsgClient {
            inner,
        })
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
    pub fn msg(&mut self, msg: impl Into<Any>) {
        self.inner.msg(msg);
    }

    /// Adds multiple Msgs to the transaction
    pub fn msgs(&mut self, msgs: impl IntoIterator<Item = Any>) {
        self.inner.msgs(msgs);
    }

    /// Adds an extension option
    pub fn extension_option(&mut self, value: impl Into<Any>) {
        self.inner.extension_option(value);
    }

    /// Adds a non-critical extension option
    pub fn non_critical_extension_option(&mut self, value: impl Into<Any>) {
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
        signer: AccountInfo,
        fee_info: FeeInfo,
        chain_context: Context,
        mut qclient: QueryClient,
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
        signer: AccountInfo,
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
    /// Broadcasts transaction using the /broadcast_commit Tendermint endpoint, waiting for CheckTx to complete before returning.
    pub async fn broadcast(self, client: &mut MsgClient) -> Result<Response> {
        match self.inner.broadcast_commit(client.inner()).await {
            Ok(response) => Ok(response),
            Err(err) => return Err(eyre!("Failed to broadcast tx: {}", err)).into(),
        }
    }

    /// Converts to the inner [`Raw`]
    pub fn into_inner(self) -> Raw {
        self.inner
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

/// Represents an arbitrary Cosmos module Msg
pub trait ModuleMsg<'m> {
    type Error;

    fn into_any(self) -> Result<Any, Self::Error>;
    fn into_tx(self) -> Result<UnsignedTx, Self::Error>;
}
