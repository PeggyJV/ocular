//! Cryptographic utilities
use bip32::{Language, Mnemonic};
use pkcs8::rand_core::OsRng;

/// Generates a 24 word English mnemonic phrase
pub fn generate_mnemonic() -> String {
    let mnemonic = Mnemonic::random(OsRng, Language::English);
    mnemonic.phrase().to_string()
}
