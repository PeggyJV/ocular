#![allow(clippy::too_many_arguments)]
//! Ocular is a client library for Cosmos SDK chains with a focus on pleasant UX. Think of it as a convenience wrapper around the [cosmrs](https://docs.rs/cosmrs/latest/cosmrs/) client features.
pub extern crate cosmrs as cosmrs;

#[cfg(feature = "query")]
pub use crate::query::QueryClient;

pub mod account;
pub mod prelude;
pub mod rpc;

#[cfg(feature = "query")]
pub mod query;

#[cfg(feature = "tx")]
pub mod chain;
#[cfg(feature = "tx")]
pub mod tx;
