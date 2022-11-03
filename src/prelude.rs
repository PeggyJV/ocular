//! Crate prelude fore easy importing
pub use crate::account::AccountInfo;

// #[cfg(feature = "query")]
// #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
// pub use query::*;

// #[cfg(feature = "query")]
// #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
// /// Prelude submodule for only importing items relevant to querying
// pub mod query {
//     pub use crate::query::{
//         auth::*, authz::*, bank::*, distribution::*, evidence::*, feegrant::*, gov::*, mint::*,
//         params::*, slashing::*, staking::*, *,
//     };
// }

// #[cfg(feature = "tx")]
// #[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
// pub use tx::*;

// #[cfg(feature = "tx")]
// #[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
// /// Prelude submodule for only importing items relevant to transactions
// pub mod tx {
//     pub use crate::chain;
//     pub use crate::tx::{
//         authz::*, bank::*, crisis::*, distribution::*, evidence::*, feegrant::*, gov::*,
//         slashing::*, staking::*, *,
//     };
// }
