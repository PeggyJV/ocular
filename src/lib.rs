#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![allow(clippy::too_many_arguments)]
//! Ocular is a client library for Cosmos SDK chains with a focus on pleasant UX. Think of it as a convenience wrapper around the [cosmrs](https://docs.rs/cosmrs/latest/cosmrs/) client features.
pub extern crate cosmrs as cosmrs;

/// Convenience alias for Tendermint RPC HTTP client type
pub type HttpClient = crate::cosmrs::rpc::HttpClient;

#[cfg(feature = "query")]
pub use crate::query::QueryClient;

pub mod account;
pub mod prelude;

#[cfg(feature = "query")]
pub mod query;

#[cfg(feature = "tx")]
#[cfg_attr(docsrs, doc(cfg(feature = "tx")))]
pub mod chain;
#[cfg(feature = "tx")]
#[cfg_attr(docsrs, doc(cfg(feature = "tx")))]
pub mod tx;
