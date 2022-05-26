use cosmrs::{tendermint::chain::Id, Coin};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BatchToml<'a> {
    pub sender: BatchSender<'a>,
    #[serde(borrow)]
    pub transactions: Vec<BatchTransaction<'a>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BatchSender<'a> {
    pub source_private_key_path: &'a str,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BatchTransaction<'a> {
    pub name: &'a str,
    pub destination_account: &'a str,
    pub amount: u64,
    pub denom: &'a str,
    pub gas_limit: u64,
    pub gas_fee: u64,
    pub timeout_height: u32,
    pub memo: &'a str,
}

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
