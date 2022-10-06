//! Types providing context and references to specific Cosmos chains

/// Provides chain-specific context
#[derive(Clone, Debug)]
pub struct ChainContext {
    /// The chain's ID, usually formatted as <chain_name>-<major_version_number> (ex. cosmoshub-4)
    pub id: String,
    /// The account prefix used for the chain's bech32 address representations
    pub prefix: String,
}
