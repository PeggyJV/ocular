use crate::error::TransactionError;
use cosmrs::{
    bank::MsgSend,
    crypto::{secp256k1::SigningKey, PublicKey},
    staking::{MsgDelegate, MsgUndelegate},
    tendermint::chain::Id,
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use super::ChainClient;

/// Metadata wrapper for transactions
pub struct TransactionMetadata {
    pub chain_id: Id,
    pub account_number: u64,
    pub sequence_number: u64,
    pub gas_limit: u64,
    pub timeout_height: u16,
    pub memo: String,
}

impl ChainClient {
    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Helper method for signing and broadcasting messages.
    async fn sign_and_send_msg(
        &self,
        sender_public_key: PublicKey,
        sender_private_key: SigningKey,
        amount: Coin,
        tx_body: tx::Body,
        tx_metadata: TransactionMetadata,
    ) -> Result<Response, TransactionError> {
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

        // Broadcast transaction
        match tx_signed.broadcast_commit(&self.rpc_client).await {
            Ok(response) => Ok(response),
            Err(err) => Err(TransactionError::BroadcastError(err.to_string())),
        }
    }

    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Signs and sends a simple transaction message.
    pub async fn sign_and_send_msg_send(
        &self,
        sender_account: AccountId,
        sender_public_key: PublicKey,
        sender_private_key: SigningKey,
        recipient_account: AccountId,
        amount: Coin,
        tx_metadata: TransactionMetadata,
    ) -> Result<Response, TransactionError> {
        // Create send message for amount
        let msg = MsgSend {
            from_address: sender_account,
            to_address: recipient_account,
            amount: vec![amount.clone()],
        };

        // Build tx body.
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TransactionError::SerializationError(err.to_string())),
        };

        self.sign_and_send_msg(
            sender_public_key,
            sender_private_key,
            amount,
            tx_body,
            tx_metadata,
        ).await
    }

    /// Sign and send delegate message
    pub async fn sign_and_send_msg_delegate(
        &self,
        delegator_account: AccountId,
        delegator_public_key: PublicKey,
        delegator_private_key: SigningKey,
        validator_account: AccountId,
        amount: Coin,
        tx_metadata: TransactionMetadata,
    ) -> Result<Response, TransactionError> {
        // Create delegate message for amount
        let msg = MsgDelegate {
            delegator_address: delegator_account,
            validator_address: validator_account,
            amount: amount.clone(),
        };

        // Build tx body.
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TransactionError::SerializationError(err.to_string())),
        };

        self.sign_and_send_msg(
            delegator_public_key,
            delegator_private_key,
            amount,
            tx_body,
            tx_metadata,
        ).await
    }

    /// Sign and send undelegate message
    pub async fn sign_and_send_msg_undelegate(
        &self,
        delegator_account: AccountId,
        delegator_public_key: PublicKey,
        delegator_private_key: SigningKey,
        validator_account: AccountId,
        amount: Coin,
        tx_metadata: TransactionMetadata,
    ) -> Result<Response, TransactionError> {
        // Create undelegate message for amount
        let msg = MsgUndelegate {
            delegator_address: delegator_account,
            validator_address: validator_account,
            amount: amount.clone(),
        };

        // Build tx body.
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TransactionError::SerializationError(err.to_string())),
        };

        self.sign_and_send_msg(
            delegator_public_key,
            delegator_private_key,
            amount,
            tx_body,
            tx_metadata,
        ).await
    }
}

// Disclaimer on testing: Since the above commands inherently require chains to operate, testing is deferred to integration tests in ocular/tests/single_node_chain_txs.rs

