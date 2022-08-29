use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cosmrs::{tx::BodyBuilder, AccountId, Any, Coin};

/// Metadata wrapper for transactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeeInfo {
    pub fee: Coin,
    pub fee_payer: Option<AccountId>,
    pub fee_granter: Option<AccountId>,
    pub gas_limit: u64,
}

#[derive(Clone, Debug)]
pub struct UnsignedTx {
    inner: BodyBuilder,
}

impl UnsignedTx {
    /// Add a Msg to the transaction
    pub fn msg(&mut self, msg: impl Into<Any>) {
        self.inner.msg(msg);
    }

    /// Add multiple Msgs to the transaction
    pub fn msgs(&mut self, msgs: impl IntoIterator<Item = Any>) {
        self.inner.msgs(msgs);
    }

    /// Add an extension option
    pub fn extension_option(&mut self, value: impl Into<Any>) {
        self.inner.extension_option(value);
    }

    /// Add a non-critical extension option
    pub fn non_critical_extension_option(&mut self, value: impl Into<Any>) {
        self.inner.non_critical_extension_option(value);
    }

    /// Add a memo
    pub fn memo(&mut self, value: impl Into<String>) {
        self.inner.memo(value);
    }

    /// Add a timeout height
    pub fn timeout_height(&mut self, value: u32) {
        self.inner.timeout_height(value);
    }

    /// Converts to the inner [`BodyBuilder`]
    pub fn into_inner(self) -> BodyBuilder {
        self.inner
    }
}

impl From<BodyBuilder> for UnsignedTx {
    fn from(builder: BodyBuilder) -> Self {
        UnsignedTx { inner: builder }
    }
}

/// Represents an arbitrary Cosmos module Msg
pub trait ModuleMsg<'m> {
    type Error;

    fn into_any(self) -> Result<Any, Self::Error>;
    fn into_tx(self) -> Result<UnsignedTx, Self::Error>;
}
