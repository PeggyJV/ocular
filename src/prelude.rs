//! Crate prelude fore easy importing
pub use crate::account::AccountInfo;

#[cfg(feature = "grpc")]
#[cfg_attr(docsrs, doc(cfg(feature = "grpc")))]
pub use grpc::*;

#[cfg(feature = "grpc")]
#[cfg_attr(docsrs, doc(cfg(feature = "grpc")))]
/// Prelude submodule for only importing items relevant to querying
pub mod grpc {
    pub use crate::grpc::{
        auth::*, authz::*, bank::*, crisis::*, distribution::*, evidence::*, feegrant::*, gov::*,
        mint::*, params::*, slashing::*, staking::*, tendermint::*, tx::*, *,
    };
}

pub use crate::tx::*;
