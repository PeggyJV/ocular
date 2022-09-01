//! Types providing context and references to specific Cosmos chains

/// Provides chain-specific context
#[derive(Clone, Debug)]
pub struct Context {
    pub id: String,
    pub prefix: String,
}
