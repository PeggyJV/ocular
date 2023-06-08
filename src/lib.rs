#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![allow(clippy::too_many_arguments)]
//! Ocular is a client library for Cosmos SDK chains with a focus on pleasant UX. Think of it as a convenience wrapper around the [cosmrs](https://docs.rs/cosmrs/latest/cosmrs/) client features.
pub extern crate cosmrs as cosmrs;

/// Convenience alias for Tendermint RPC HTTP client type
pub type HttpClient = crate::cosmrs::rpc::HttpClient;
pub use crate::grpc::GrpcClient;

pub mod account;
pub mod chain;

#[cfg(feature = "crypto")]
#[cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
pub mod crypto;

pub mod grpc;
pub mod prelude;
pub mod tx;
