# Ocular

Ocular seeks to be the preferred cosmos client library UX for Rust projects. It is strongly based on [Ocular](https://github.com/strangelove-ventures/Ocular), a go client library for blockchains built with the [Cosmos SDK](https://github.com/cosmos/cosmos-sdk).

# Features

## Existing
- Automatic client configuration based on the [cosmos chain registry](https://github.com/cosmos/chain-registry):
```
let client = ChainClient::new("cosmoshub");
```
## In Progress
- Core module querys
- Tendermint querys
- Key management
- TX signing with the familiar cosmos Accounts paradigm
- Automatic IBC relayer path retrieval

## Future
- Arbitrary gRPC message sending (for custom modules)
