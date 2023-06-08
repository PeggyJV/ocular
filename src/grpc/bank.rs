//! Queries and messages for the [Bank module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/bank/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`BankQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{
    bank::{MsgMultiSend, MsgSend, MultiSendIo},
    tx::Msg,
    AccountId, Any, Coin, Denom,
};
use eyre::{eyre, Context, Report, Result};
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::bank::v1beta1::{
        self as bank, QueryAllBalancesResponse, QueryBalanceResponse,
        QuerySpendableBalancesResponse, QueryTotalSupplyResponse,
    },
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest, PageResponse};

/// The bank module's query client proto definition
pub type BankQueryClient = bank::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<BankQueryClient> for BankQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        BankQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets a balance at the specified address. If the balance doesn't exist, returns a [`Coin`] with an amount of zero.
    pub async fn query_balance(&mut self, address: &str, denom: &str) -> Result<BalanceResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QueryBalanceRequest {
            address: address.to_string(),
            denom: denom.to_string(),
        };

        query_client.balance(request).await?.into_inner().try_into()
    }

    /// Gets all coin balances of the specified address with optional pagination
    pub async fn query_all_balances(&mut self, address: &str) -> Result<CoinsResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QueryAllBalancesRequest {
            address: address.to_string(),
            pagination: None,
        };

        query_client
            .all_balances(request)
            .await?
            .into_inner()
            .try_into()
    }

    /// Gets spendable coin balances of the specified address with optional pagination
    pub async fn query_spendable_balances(&mut self, address: &str) -> Result<CoinsResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QuerySpendableBalancesRequest {
            address: address.to_string(),
            pagination: None,
        };

        query_client
            .spendable_balances(request)
            .await?
            .into_inner()
            .try_into()
    }

    /// Gets the bank module's params
    pub async fn query_bank_params(&mut self) -> Result<bank::QueryParamsResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }

    /// Gets metadata for the specified coin denomination if it exists, errors otherwise
    pub async fn query_denom_metadata(
        &mut self,
        denom: &str,
    ) -> Result<bank::QueryDenomMetadataResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomMetadataRequest {
            denom: denom.to_string(),
        };

        Ok(query_client.denom_metadata(request).await?.into_inner())
    }

    /// Gets the metadata for all coin denominations defined in the bank module.
    pub async fn query_all_denoms_metadata(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<bank::QueryDenomsMetadataResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomsMetadataRequest { pagination };

        Ok(query_client.denoms_metadata(request).await?.into_inner())
    }

    /// Gets the supply of the specified coin denomination
    pub async fn query_supply_of(&mut self, denom: &str) -> Result<Coin> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QuerySupplyOfRequest {
            denom: denom.to_string(),
        };
        let response = query_client.supply_of(request).await?.into_inner();

        match response.amount {
            Some(a) => Ok(a.try_into()?),
            None => Err(eyre!(
                "Empty denom supply result. Denom {denom} is probably invalid!"
            )),
        }
    }

    /// Gets the supply of all coin denominations with optional pagination
    pub async fn query_total_supply(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<CoinsResponse> {
        let query_client = self.get_client::<BankQueryClient>().await?;
        let request = bank::QueryTotalSupplyRequest { pagination };

        query_client
            .total_supply(request)
            .await?
            .into_inner()
            .try_into()
    }
}

/// Convenience type for representing the queried balance with [`cosmrs::Coin`]
#[derive(Clone, Debug)]
pub struct BalanceResponse {
    /// Coin balance. [`None`] if the account has no balance in the request denomination.
    pub balance: Option<Coin>,
}

impl TryFrom<QueryBalanceResponse> for BalanceResponse {
    type Error = Report;

    fn try_from(response: QueryBalanceResponse) -> Result<Self> {
        Ok(match response.balance {
            Some(b) => BalanceResponse {
                balance: Some(b.try_into()?),
            },
            None => BalanceResponse { balance: None },
        })
    }
}

/// Convenience type for representing the queried balances with [`cosmrs::Coin`]
#[derive(Clone, Debug)]
pub struct CoinsResponse {
    /// Total balances of each coin denomination in the chain
    pub balances: Vec<Coin>,
    /// Paging info
    pub pagination: Option<PageResponse>,
}

impl TryFrom<QueryAllBalancesResponse> for CoinsResponse {
    type Error = Report;

    fn try_from(response: QueryAllBalancesResponse) -> Result<Self> {
        let mut balances = Vec::<Coin>::new();
        for b in response.balances {
            balances.push(b.try_into()?)
        }

        Ok(Self {
            balances,
            pagination: response.pagination,
        })
    }
}

impl TryFrom<QuerySpendableBalancesResponse> for CoinsResponse {
    type Error = Report;

    fn try_from(response: QuerySpendableBalancesResponse) -> Result<Self> {
        let mut balances = Vec::<Coin>::new();
        for b in response.balances {
            balances.push(b.try_into()?)
        }

        Ok(Self {
            balances,
            pagination: response.pagination,
        })
    }
}

impl TryFrom<QueryTotalSupplyResponse> for CoinsResponse {
    type Error = Report;

    fn try_from(response: QueryTotalSupplyResponse) -> Result<Self> {
        let mut balances = Vec::<Coin>::new();
        for b in response.supply {
            balances.push(b.try_into()?)
        }

        Ok(Self {
            balances,
            pagination: response.pagination,
        })
    }
}

/// Represents a [Bank module message](https://docs.cosmos.network/v0.45/modules/bank/03_messages.html)
#[derive(Clone, Debug)]
pub enum Bank<'m> {
    /// Send coins from one account to another. Represents a [`MsgSend`]
    Send {
        /// Address of the sender account
        from: &'m str,
        /// Address of the recipient account
        to: &'m str,
        /// Amount to send
        amount: u128,
        /// Coin denomination to send
        denom: &'m str,
    },
    /// Send multiple denominations of coins from one account to another. Represents a [`MsgSend`]
    SendMultipleDenoms {
        /// Address of the sender account
        from: &'m str,
        /// Address of the recipient account
        to: &'m str,
        /// Coins to send
        amount: Vec<Coin>,
    },
    /// Send coins from one or more accounts to one or more accounts. Note that each input account must be
    /// a signer of the transaction, and the sum of the input coins must equal the sum of output coins. To learn more,
    /// see https://docs.cosmos.network/master/modules/bank/03_messages.html#msgmultisend. Represents a [`MsgMultiSend`]
    MultiSend {
        /// Sending address/amount inputs
        inputs: Vec<MultiSendIo>,
        /// Receiving address/amount inputs
        outputs: Vec<MultiSendIo>,
    },
}

impl ModuleMsg for Bank<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Bank::Send {
                from,
                to,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };
                MsgSend {
                    from_address: AccountId::from_str(from)?,
                    to_address: AccountId::from_str(to)?,
                    amount: vec![amount],
                }
                .to_any()
            }
            Bank::SendMultipleDenoms { from, to, amount } => MsgSend {
                from_address: AccountId::from_str(from)?,
                to_address: AccountId::from_str(to)?,
                amount,
            }
            .to_any(),
            Bank::MultiSend { inputs, outputs } => MsgMultiSend { inputs, outputs }.to_any(),
        }
    }

    /// Converts the message enum representation into an [`UnsignedTx`] containing the corresponding Msg
    fn into_tx(self) -> Result<UnsignedTx> {
        let mut tx = UnsignedTx::new();
        tx.add_msg(self.into_any()?);

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_txs() {
        Bank::Send {
            to: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            from: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Bank::MultiSend {
            inputs: vec![],
            outputs: vec![],
        }
        .into_tx()
        .unwrap();
    }
}
