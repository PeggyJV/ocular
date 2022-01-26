# Ocular

Ocular is (will be) a rust implementation of [lens](https://github.com/strangelove-ventures/lens), a client library and associated CLI for blockchains built with the [Cosmos SDK](https://github.com/cosmos/cosmos-sdk).

## To do
This list is not exhaustive and will be updated periodically. If you are interested in contributing, compare with the [lens](https://github.com/strangelove-ventures/lens) repo to see what is missing.

`ocular`
- [X] Chain registry interaction
- [X] Structs for deserialized chain info from `chain.json`
- [X] Structs for deserialized assets info from `assets.json`
- [ ] Chain client config struct (in progress)

`ocular_cli`
- [X] Chain info serializaton to TOML
- [X] Config files initialization
- [ ] Implement `chains` subcommands
    - [X] `show`
    - [ ] `add`
    - [ ] `update`
    - [ ] `delete`
    - [ ] `edit`
    - [ ] `editor`
    - [ ] `list`
    - [ ] `registry_list`
    - [ ] `show_default`
    - [ ] `set_default`
