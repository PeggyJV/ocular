use crate::{
    account::{Account, BaseAccount},
    error::{ChainClientError, TxError},
    tx::{TxMetadata, MultiSendIO},
};
use cosmrs::{
    bank::{MsgSend, MsgMultiSend},
    crypto::secp256k1::SigningKey,
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::{fs::File, str::FromStr};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use super::ChainClient;

/// Where tx logs are stored.
const TX_LOGGING_DIR: &str = "/.ocular/logs/txs";
/// Unix permissions for dir
const TX_LOGGING_DIR_PERMISSIONS: u32 = 0o700;

impl ChainClient {
    pub async fn get_basic_tx_metadata(&self) -> Result<TxMetadata, ChainClientError> {
        let current_height = self.query_latest_height().await?;
        let timeout_height: u32 = (current_height + 30) as u32;
        let fee = self.config.default_fee.clone();

        Ok(TxMetadata {
            fee,
            gas_limit: 200000,
            timeout_height,
            memo: String::default(),
        })
    }

    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Helper method for signing and broadcasting messages.
    pub async fn sign_and_send_msg(
        &self,
        sender: BaseAccount,
        signer: SigningKey,
        tx_body: tx::Body,
        tx_metadata: TxMetadata,
        fee_payer: Option<AccountId>,
        fee_granter: Option<AccountId>,
    ) -> Result<Response, ChainClientError> {
        // Create signer info.
        let signer_info = SignerInfo::single_direct(Some(sender.pub_key), sender.sequence);

        // Compute auth info from signer info by associating a fee.
        let auth_info = signer_info.auth_info(Fee {
            amount: vec![tx_metadata.fee.try_into()?],
            gas_limit: tx_metadata.gas_limit.into(),
            payer: fee_payer,
            granter: fee_granter,
        });
        let chain_id = &cosmrs::tendermint::chain::Id::try_from(self.config.chain_id.clone())
            .expect(format!("failed to create chain ID from {}", self.config.chain_id).as_str());

        // Create doc to be signed
        let sign_doc = match SignDoc::new(&tx_body, &auth_info, chain_id, sender.account_number) {
            Ok(doc) => doc,
            Err(err) => return Err(TxError::TypeConversion(err.to_string()).into()),
        };

        // Create raw signed transaction.
        let tx_signed = match sign_doc.sign(&signer) {
            Ok(raw) => raw,
            Err(err) => return Err(TxError::Signing(err.to_string()).into()),
        };

        // Broadcast transaction
        let response = match tx_signed.broadcast_commit(&self.rpc_client).await {
            Ok(response) => response,
            Err(err) => return Err(TxError::Broadcast(err.to_string()).into()),
        };

        // Store tx in logs with timestamp id in ~/.ocular/logs/txs
        let save_path = dirs::home_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .expect("Could not obtain home directory.")
            + TX_LOGGING_DIR;

        let save_file = save_path.clone()
            + "/"
            + &std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
            + &std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
                .to_string()
            + ".txt";

        let st = std::path::Path::new(&save_path).metadata();

        // Create dir if doesn't exist
        if st.is_err() {
            match std::fs::create_dir_all(&save_path) {
                Ok(_res) => _res,
                Err(err) => return Err(TxError::Logging(err.to_string()).into()),
            };
        }

        #[cfg(unix)]
        match std::fs::set_permissions(
            &save_path,
            std::fs::Permissions::from_mode(TX_LOGGING_DIR_PERMISSIONS),
        ) {
            Ok(_res) => _res,
            Err(err) => return Err(TxError::Logging(err.to_string()).into()),
        };

        let mut file = match File::create(save_file) {
            Ok(res) => res,
            Err(err) => return Err(TxError::Logging(err.to_string()).into()),
        };

        match file.write_all(format!("{:#?}", response).as_bytes()) {
            Ok(res) => res,
            Err(err) => return Err(TxError::Logging(err.to_string()).into()),
        };

        // Finally return.
        Ok(response)
    }

    // TODO: Make this extensible to multisig and multicoin (or add new methods for that)
    /// Signs and sends a simple transaction message.
    pub async fn send(
        &self,
        sender: Account,
        recipient: &str,
        amount: Coin,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let signer = sender.private_key;
        let recipient = match AccountId::from_str(recipient) {
            Ok(r) => r,
            Err(err) => {
                return Err(TxError::Address(format!(
                    "failed to get AccountId from string {}: {}",
                    recipient,
                    err
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
            from_address: sender.id.clone(),
            to_address: recipient,
            amount: vec![amount.clone()],
        };
        let sender = self.query_account(sender.id.as_ref().to_string()).await?;
        let sender = BaseAccount::try_from(sender)?;
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TxError::Serialization(err.to_string()).into()),
        };

        self.sign_and_send_msg(sender, signer, tx_body, tx_metadata, None, None)
            .await
    }

    pub async fn multi_send(
        &self,
        sender: Account,
        inputs: Vec<MultiSendIO>,
        outputs: Vec<MultiSendIO>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgMultiSend {
            inputs: inputs.iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            outputs: outputs.iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
        };
        let signer = sender.private_key;
        let sender = self.query_account(sender.id.as_ref().to_string()).await?;
        let sender = BaseAccount::try_from(sender)?;
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = match msg.to_any() {
            Ok(msg) => tx::Body::new(vec![msg], &tx_metadata.memo, tx_metadata.timeout_height),
            Err(err) => return Err(TxError::Serialization(err.to_string()).into()),
        };

        self.sign_and_send_msg(sender, signer, tx_body, tx_metadata, None, None).await
    }
}
