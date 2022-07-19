use std::str::FromStr;

use cosmrs::{
    bank::{MsgMultiSend, MsgSend},
    tx::{self, Msg},
    AccountId, Coin,
};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use crate::{
    account::AccountInfo,
    error::{ChainClientError, TxError},
    tx::{MultiSendIo, TxMetadata},
};

use super::ChainClient;

impl ChainClient {
    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Signs and sends a simple transaction message.
    pub async fn send(
        &mut self,
        sender: &AccountInfo,
        recipient: &str,
        amount: Coin,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let recipient = match AccountId::from_str(recipient) {
            Ok(r) => r,
            Err(err) => {
                return Err(TxError::Address(format!(
                    "failed to get AccountId from string {}: {}",
                    recipient, err
                ))
                .into())
            }
        };

        if recipient.prefix() != self.config.account_prefix {
            return Err(TxError::Address(format!(
                "invalid address prefix. expected {}, got {}",
                self.config.account_prefix,
                recipient.prefix()
            ))
            .into());
        }

        let msg = MsgSend {
            from_address: sender.id(&self.config.account_prefix)?,
            to_address: recipient,
            amount: vec![amount.clone()],
        };
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TxError::Serialization(err.to_string()).into()),
        };

        self.sign_and_send_msg(sender, tx_body, tx_metadata).await
    }

    /// Send coins in a MIMO fashion. If any coin transfers are invalid the entire transaction will fail.
    pub async fn multi_send(
        &mut self,
        sender: &AccountInfo,
        inputs: Vec<MultiSendIo>,
        outputs: Vec<MultiSendIo>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgMultiSend {
            inputs: inputs
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            outputs: outputs
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        };
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TxError::Serialization(err.to_string()).into()),
        };

        self.sign_and_send_msg(sender, tx_body, tx_metadata).await
    }
}
