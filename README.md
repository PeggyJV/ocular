# Ocular

Ocular is a client library for Cosmos SDK chains with a focus on pleasent UX. Think of it as a convenience wrapper around the [cosmrs](https://docs.rs/cosmrs/latest/cosmrs/) client features.

# Beta Features

- `QueryClient` providing an exhaustive API for core SDK module queries
- `MsgClient` providing and API for core SDK module Msgs (in progress)
- Robust transaction construction allowing multiple Msgs in one tx
- `ModuleMsg` trait for creating extension crates to easily support custom Cosmos modules.
- Convenient `AccountInfo` type constructable from a mnemonic or on-disk key. Used for signing and deriving addresses for various chains.

## To do

- (Nice-to-have) Load chain context from chain registry
- Key generation/persistance API

# Contributions

Feel free to fork and PR! DM Collin on Twitter @ atro0o for feedback/questions.
