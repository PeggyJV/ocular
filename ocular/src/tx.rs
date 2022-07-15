pub mod bank;

use std::str::FromStr;

use cosmrs::{
    tx::{Body, Fee, Raw, SignDoc, SignerInfo},
    AccountId, Any, Denom,
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;

use crate::{account::AccountInfo, chain::client::ChainClient, error::TxError};

#[derive(Debug)]
pub struct SignedTx {
    inner: Raw,
}

impl SignedTx {
    /// Broadcasts transaction using the /broadcast_commit Tendermint endpoint, waiting for CheckTx to complete before returning.
    pub async fn broadcast(self, client: &mut ChainClient) -> Result<Response, TxError> {
        match self.inner.broadcast_commit(&client.rpc_client).await {
            Ok(response) => Ok(response),
            Err(err) => return Err(TxError::Broadcast(err.to_string()).into()),
        }
    }

    pub fn into_inner(self) -> Raw {
        self.inner
    }
}

#[derive(Clone, Debug)]
pub struct UnsignedTx {
    messages: Vec<Any>,
    metadata: TxMetadata,
}

impl UnsignedTx {
    pub fn fee(&mut self, value: Coin) {
        self.metadata.fee = value;
    }

    pub fn fee_granter(&mut self, value: &str) -> Result<(), TxError> {
        self.metadata.fee_granter = Some(AccountId::from_str(value)?);

        Ok(())
    }

    pub fn fee_payer(&mut self, value: &str) -> Result<(), TxError> {
        self.metadata.fee_payer = Some(AccountId::from_str(value)?);

        Ok(())
    }

    pub fn gas_limit(&mut self, value: u64) {
        self.metadata.gas_limit = value;
    }

    pub fn timeout_height(&mut self, value: u32) {
        self.metadata.timeout_height = value;
    }

    pub async fn sign(
        self,
        client: &mut ChainClient,
        signer: &AccountInfo,
    ) -> Result<SignedTx, TxError> {
        let account = client
            .query_account(
                &signer
                    .address(&client.config.account_prefix)
                    .map_err(|e| TxError::Address(e.to_string()))?,
            )
            .await
            .map_err(|e| TxError::Account(e.to_string()))?;
        let signer_info = SignerInfo::single_direct(Some(signer.public_key()), account.sequence);
        let auth_info = signer_info.auth_info(Fee {
            amount: vec![self.metadata.fee.try_into()?],
            gas_limit: self.metadata.gas_limit.into(),
            payer: self.metadata.fee_payer,
            granter: self.metadata.fee_granter,
        });
        let chain_id = &cosmrs::tendermint::chain::Id::try_from(client.config.chain_id.clone())?;
        let tx_body = Body::new(
            self.messages,
            self.metadata.memo,
            self.metadata.timeout_height,
        );
        let sign_doc = match SignDoc::new(&tx_body, &auth_info, chain_id, account.account_number) {
            Ok(doc) => doc,
            Err(err) => return Err(TxError::TypeConversion(err.to_string()).into()),
        };
        let tx_signed = match sign_doc.sign(signer.private_key()) {
            Ok(raw) => raw,
            Err(err) => return Err(TxError::Signing(err.to_string()).into()),
        };

        Ok(SignedTx { inner: tx_signed })
    }
}

/// Metadata wrapper for transactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxMetadata {
    pub fee: Coin,
    pub fee_payer: Option<AccountId>,
    pub fee_granter: Option<AccountId>,
    pub gas_limit: u64,
    pub timeout_height: u32,
    #[serde(default)]
    pub memo: String,
}

impl Default for TxMetadata {
    fn default() -> Self {
        Self {
            gas_limit: 200000,
            fee: Coin::default(),
            fee_payer: None,
            fee_granter: None,
            timeout_height: 0,
            memo: String::default(),
        }
    }
}

pub trait Module<'m> {
    type Error;

    fn try_into_tx(self) -> Result<UnsignedTx, Self::Error>;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Coin {
    pub amount: u64,
    pub denom: String,
}

impl TryFrom<Coin> for cosmrs::Coin {
    type Error = TxError;

    fn try_from(coin: Coin) -> Result<cosmrs::Coin, Self::Error> {
        cosmrs::Coin::try_from(&coin)
    }
}

impl TryFrom<&Coin> for cosmrs::Coin {
    type Error = TxError;

    fn try_from(coin: &Coin) -> Result<cosmrs::Coin, Self::Error> {
        Ok(cosmrs::Coin {
            denom: coin.denom.parse::<Denom>()?,
            amount: coin.amount.into(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Payment {
    pub recipient: String,
    pub amount: u64,
    pub denom: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PaymentsToml {
    pub sender_key_name: String,
    pub grantee_key_name: Option<String>,
    pub fee_granter: Option<String>,
    pub fee_payer: Option<String>,
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiSendIo {
    pub address: String,
    pub coins: Vec<Coin>,
}

impl TryFrom<MultiSendIo> for cosmrs::bank::MultiSendIo {
    type Error = TxError;

    fn try_from(value: MultiSendIo) -> Result<cosmrs::bank::MultiSendIo, Self::Error> {
        cosmrs::bank::MultiSendIo::try_from(&value)
    }
}

impl TryFrom<&MultiSendIo> for cosmrs::bank::MultiSendIo {
    type Error = TxError;

    fn try_from(value: &MultiSendIo) -> Result<cosmrs::bank::MultiSendIo, Self::Error> {
        Ok(cosmrs::bank::MultiSendIo {
            address: value.address.as_str().parse::<AccountId>()?,
            coins: value
                .coins
                .iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
