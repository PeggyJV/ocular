pub use crate::account::AccountInfo;

#[cfg(feature = "query")]
pub use query::*;

#[cfg(feature = "query")]
pub mod query {
    pub use crate::query::{
        auth::*, authz::*, bank::*, distribution::*, evidence::*, feegrant::*, gov::*, mint::*,
        params::*, rpc::*, slashing::*, staking::*, *,
    };
}

#[cfg(feature = "tx")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
pub use tx::*;

#[cfg(feature = "tx")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
pub mod tx {
    pub use crate::chain;
    pub use crate::tx::{authz::*, bank::*, crisis::*, distribution::*, evidence::*, feegrant::*,
        gov::*, slashing::*, staking::*, *};
}
