/// Cryptographic utilities
use cosmrs::bip32::{Mnemonic, Language};
use pkcs8::rand_core::OsRng;

/// Generates a 24 word English mnemonic phrase
pub fn generate_mnemonic() -> String {
    let mnemonic = Mnemonic::random(&mut OsRng, Language::English);
    mnemonic.phrase().to_string()
}
