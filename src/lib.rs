#![allow(clippy::too_many_arguments)]
//! Ocular is a client library for Cosmos SDK chains with a focus on pleasant UX. Think of it as a convenience wrapper around the [cosmrs](https://docs.rs/cosmrs/latest/cosmrs/) client features.
pub extern crate cosmrs as cosmrs;

pub use query::QueryClient;

pub mod account;
pub mod query;
pub mod rpc;
