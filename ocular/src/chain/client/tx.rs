use crate::{
    account::AccountInfo,
    error::{ChainClientError, TxError},
    tx::TxMetadata,
};
use cosmrs::tx::{self, Fee, SignDoc, SignerInfo};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use super::ChainClient;

pub use self::{authz::*, bank::*};

pub mod authz;
pub mod bank;

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
            fee_payer: None,
            fee_granter: None,
            gas_limit: 200000,
            timeout_height,
            memo: String::default(),
        })
    }

    /// Helper method for signing and broadcasting messages.
    pub async fn sign_and_send_msg(
        &mut self,
        sender: AccountInfo,
        tx_body: tx::Body,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let account = self.query_account(sender.id.as_ref().to_string()).await?;

        // Create signer info.
        let signer_info = SignerInfo::single_direct(Some(sender.public_key), account.sequence);

        // Compute auth info from signer info by associating a fee.
        let auth_info = signer_info.auth_info(Fee {
            amount: vec![tx_metadata.fee.try_into()?],
            gas_limit: tx_metadata.gas_limit.into(),
            payer: tx_metadata.fee_payer,
            granter: tx_metadata.fee_granter,
        });
        let chain_id = &cosmrs::tendermint::chain::Id::try_from(self.config.chain_id.clone())?;

        // Create doc to be signed
        let sign_doc = match SignDoc::new(&tx_body, &auth_info, chain_id, account.account_number) {
            Ok(doc) => doc,
            Err(err) => return Err(TxError::TypeConversion(err.to_string()).into()),
        };

        // Create raw signed transaction.
        let tx_signed = match sign_doc.sign(&sender.private_key) {
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
}
