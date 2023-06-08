# Ocular

Ocular is a gRPC client library for Cosmos SDK chains with a focus on pleasent UX. Think of it as a convenience wrapper around the [cosmrs](https://docs.rs/cosmrs/latest/cosmrs/) client features.

# Beta Features

- `QueryClient` providing an API for core Cosmos SDK module queries
- `MsgClient` providing an API for executing transactions, and support for core Cosmos SDK module messages
- Robust transaction construction allowing multiple Msgs in one tx
- `ModuleMsg` trait for creating extension crates to easily support custom Cosmos modules.
- Convenient `AccountInfo` type constructable from a mnemonic or on-disk key. Used for signing and deriving addresses for various chains.

## To do

- RPC?
- Feature enabling dynamic chain contexts populated by chain registry data
- Key generation/persistance API
- Websocket?
- Code examples
- Convenience wrappers around proto-generated types (like authz::{Grant, GrantAuthorization, GenericAuthorization})

# Contributions

Feel free to fork and PR! DM Collin on Twitter @ atro0o for feedback/questions.
