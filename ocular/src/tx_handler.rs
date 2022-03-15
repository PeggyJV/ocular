use crate::error::TransactionError;
use cosmrs::{
    bank::MsgSend,
    crypto::{secp256k1::SigningKey, PublicKey},
    tendermint::chain::Id,
    tx::{self, Fee, Msg, SignDoc, SignerInfo, Tx},
    AccountId, Coin,
};

/// Transaction Handler off of which all transactions can be performed from.
pub struct TransactionHandler {}

impl TransactionHandler {
    /// Creates and signs a send message. Returns an error if unsuccessful in creating a Tx.
    pub fn create_and_sign_send_message(
        sender_account: AccountId,
        sender_public_key: PublicKey,
        sender_private_key: SigningKey,
        recipient_account: AccountId,
        amount: Coin,
        chain_id: &Id,
        account_number: u64,
        sequence_number: u64,
        gas_limit: u64,
        timeout_height: u16,
        memo: &str,
    ) -> Result<Tx, TransactionError> {
        // Create send message for coin
        let msg_send = MsgSend {
            from_address: sender_account.clone(),
            to_address: recipient_account,
            amount: vec![amount.clone()],
        };

        // Build tx body.
        let tx_body = match msg_send.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], memo, timeout_height),
            Err(err) => return Err(TransactionError::SerializationError(err.to_string())),
        };

        // Create signer info.
        let signer_info = SignerInfo::single_direct(Some(sender_public_key), sequence_number);

        // Compute auth info from signer info by associating a fee.
        let auth_info = signer_info.auth_info(Fee::from_amount_and_gas(amount, gas_limit));

        // Create doc to be signed
        let sign_doc = match SignDoc::new(&tx_body, &auth_info, chain_id, account_number) {
            Ok(doc) => doc,
            Err(err) => return Err(TransactionError::TypeConversionError(err.to_string())),
        };

        // Create raw signed transaction.
        let tx_signed = match sign_doc.sign(&sender_private_key) {
            Ok(raw) => raw,
            Err(err) => return Err(TransactionError::SigningError(err.to_string())),
        };

        // Serialize into bytes.
        let tx_bytes = match tx_signed.to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => return Err(TransactionError::SerializationError(err.to_string())),
        };

        // Convert to cosmrs Tx
        match Tx::from_bytes(&tx_bytes) {
            Ok(tx) => Ok(tx),
            Err(err) => return Err(TransactionError::TypeConversionError(err.to_string())),
        }
    }
}

// ---------------------------------- Tests ----------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder() {
        assert_eq!(true, false);
    }
}
