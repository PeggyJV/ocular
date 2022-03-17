use crate::error::TransactionError;
use cosmrs::{
    bank::MsgSend,
    crypto::{secp256k1::SigningKey, PublicKey},
    tendermint::chain::Id,
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};

/// Transaction Handler off of which all transactions can be performed from.
pub struct TransactionHandler {}

/// Metadata for transaction
pub struct TransactionMetadata {
    pub chain_id: Id,
    pub account_number: u64,
    pub sequence_number: u64,
    pub gas_limit: u64,
    pub timeout_height: u16,
    pub memo: String,
}

impl TransactionHandler {
    /// Creates and signs a send message. Returns an error if unsuccessful in creating a
    pub fn create_and_sign_send_message(
        sender_account: AccountId,
        sender_public_key: PublicKey,
        sender_private_key: SigningKey,
        recipient_account: AccountId,
        amount: Coin,
        tx_metadata: TransactionMetadata,
    ) -> Result<Vec<u8>, TransactionError> {
        // Create send message for coin
        let msg_send = MsgSend {
            from_address: sender_account,
            to_address: recipient_account,
            amount: vec![amount.clone()],
        };

        // Build tx body.
        let tx_body = match msg_send.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TransactionError::SerializationError(err.to_string())),
        };

        // Create signer info.
        let signer_info =
            SignerInfo::single_direct(Some(sender_public_key), tx_metadata.sequence_number);

        // Compute auth info from signer info by associating a fee.
        // TODO: Add ability to specify payers and granters for fees.
        let auth_info =
            signer_info.auth_info(Fee::from_amount_and_gas(amount, tx_metadata.gas_limit));

        // Create doc to be signed
        let sign_doc = match SignDoc::new(
            &tx_body,
            &auth_info,
            &tx_metadata.chain_id,
            tx_metadata.account_number,
        ) {
            Ok(doc) => doc,
            Err(err) => return Err(TransactionError::TypeConversionError(err.to_string())),
        };

        // Create raw signed transaction.
        let tx_signed = match sign_doc.sign(&sender_private_key) {
            Ok(raw) => raw,
            Err(err) => return Err(TransactionError::SigningError(err.to_string())),
        };

        // Serialize into bytes.
        match tx_signed.to_bytes() {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(TransactionError::SerializationError(err.to_string())),
        }
    }
}

// ---------------------------------- Tests ----------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_sign_send_message() {
        // Create some keys to start off
        let sender_private_key = SigningKey::random();
        let sender_public_key = sender_private_key.public_key();
        let sender_account = sender_public_key
            .account_id("cosmos")
            .expect("Could not create sender account.");

        let recipient_account = SigningKey::random()
            .public_key()
            .account_id("cosmos")
            .expect("Could not create recipient account");

        TransactionHandler::create_and_sign_send_message(
            sender_account,
            sender_public_key,
            sender_private_key,
            recipient_account,
            Coin {
                amount: 1_000_000u64.into(),
                denom: "uatom".parse().unwrap(),
            },
            TransactionMetadata {
                chain_id: Id::try_from("cosmoshub-4").expect("Could not create chain id."),
                account_number: 1,
                sequence_number: 0,
                gas_limit: 100_000,
                timeout_height: 9001u16,
                memo: String::from("Some memo"),
            },
        )
        .expect("Could not create tx");
    }
}
