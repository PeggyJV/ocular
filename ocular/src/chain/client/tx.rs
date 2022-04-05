use crate::error::TxError;
use cosmrs::{
    bank::MsgSend,
    crypto::{secp256k1::SigningKey, PublicKey},
    tendermint::chain::Id,
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use super::ChainClient;

/// Metadata wrapper for transactions
#[derive(Clone, Debug)]
pub struct TxMetadata {
    pub chain_id: Id,
    pub account_number: u64,
    pub sequence_number: u64,
    pub gas_fee: Coin,
    pub gas_limit: u64,
    pub timeout_height: u32,
    pub memo: String,
}

///  Type to hold all information around an account.
pub struct Account {
    pub id: AccountId,
    pub public_key: PublicKey,
    pub private_key: SigningKey,
}

impl ChainClient {
    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Helper method for signing and broadcasting messages.
    pub async fn sign_and_send_msg(
        &self,
        sender_public_key: PublicKey,
        sender_private_key: SigningKey,
        tx_body: tx::Body,
        tx_metadata: TxMetadata,
        fee_payer: Option<AccountId>,
        fee_granter: Option<AccountId>,
    ) -> Result<Response, TxError> {
        // Create signer info.
        let signer_info =
            SignerInfo::single_direct(Some(sender_public_key), tx_metadata.sequence_number);

        // Compute auth info from signer info by associating a fee.
        let auth_info = signer_info.auth_info(Fee {
            amount: vec![tx_metadata.gas_fee; 1],
            gas_limit: tx_metadata.gas_limit.into(),
            payer: fee_payer,
            granter: fee_granter,
        });

        // Create doc to be signed
        let sign_doc = match SignDoc::new(
            &tx_body,
            &auth_info,
            &tx_metadata.chain_id,
            tx_metadata.account_number,
        ) {
            Ok(doc) => doc,
            Err(err) => return Err(TxError::TypeConversionError(err.to_string())),
        };

        // Create raw signed transaction.
        let tx_signed = match sign_doc.sign(&sender_private_key) {
            Ok(raw) => raw,
            Err(err) => return Err(TxError::SigningError(err.to_string())),
        };

        // Broadcast transaction
        match tx_signed.broadcast_commit(&self.rpc_client).await {
            Ok(response) => Ok(response),
            Err(err) => Err(TxError::BroadcastError(err.to_string())),
        }
    }

    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Signs and sends a simple transaction message.
    pub async fn send(
        &self,
        sender_account: Account,
        recipient_account_id: AccountId,
        amount: Coin,
        tx_metadata: TxMetadata,
    ) -> Result<Response, TxError> {
        // Create send message for amount
        let msg = MsgSend {
            from_address: sender_account.id,
            to_address: recipient_account_id,
            amount: vec![amount.clone()],
        };

        // Build tx body.
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TxError::SerializationError(err.to_string())),
        };

        self.sign_and_send_msg(
            sender_account.public_key,
            sender_account.private_key,
            tx_body,
            tx_metadata,
            None,
            None,
        )
        .await
    }
}

// Disclaimer on testing: Since the above commands inherently require chains to operate, testing is deferred to integration tests in ocular/tests/single_node_chain_txs.rs
