//! Messages for managing allowance fee grants
use std::str::FromStr;

use eyre::{Report, Result};

use crate::cosmrs::{
    feegrant::{MsgGrantAllowance, MsgRevokeAllowance},
    tx::Msg,
    AccountId, Any,
};

use super::{ModuleMsg, UnsignedTx};

/// Represents a [FeeGrant module message](https://docs.cosmos.network/v0.45/modules/feegrant/03_messages.html)
#[derive(Clone, Debug)]
pub enum FeeGrant<'m> {
    /// Grant an allowance of funds to another account to cover transaction fees. Represents a [`MsgGrantAllowance`]
    GrantAllowance {
        granter: &'m str,
        grantee: &'m str,
        allowance: Any,
    },
    /// Revoke a previously granted allowance. Represents a [`MsgRevokeAllowance`]
    RevokeAllowance { granter: &'m str, grantee: &'m str },
}

impl ModuleMsg for FeeGrant<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            FeeGrant::GrantAllowance {
                granter,
                grantee,
                allowance,
            } => MsgGrantAllowance {
                granter: AccountId::from_str(granter)?,
                grantee: AccountId::from_str(grantee)?,
                allowance: Some(allowance),
            }
            .to_any(),
            FeeGrant::RevokeAllowance { granter, grantee } => MsgRevokeAllowance {
                granter: AccountId::from_str(granter)?,
                grantee: AccountId::from_str(grantee)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        FeeGrant::GrantAllowance {
            granter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            grantee: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            allowance: Any::default(),
        }
        .into_tx()
        .unwrap();

        FeeGrant::RevokeAllowance {
            granter: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            grantee: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();
    }
}
