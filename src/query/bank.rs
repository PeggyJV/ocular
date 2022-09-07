//! Queries for the [Bank module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/bank/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`BankQueryClient`] directly.
use async_trait::async_trait;
use cosmrs::Coin;
use eyre::{eyre, Context, Report, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::bank::v1beta1::{
    self as bank, QueryAllBalancesResponse, QueryBalanceResponse, QuerySpendableBalancesResponse,
    QueryTotalSupplyResponse,
};

use super::{GrpcClient, PageRequest, PageResponse, QueryClient};

/// The bank module's query client proto definition
pub type BankQueryClient = bank::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for BankQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        BankQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Gets a balance at the specified address. If the balance doesn't exist, returns a [`Coin`] with an amount of zero.
    pub async fn balance(&mut self, address: &str, denom: &str) -> Result<BalanceResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryBalanceRequest {
            address: address.to_string(),
            denom: denom.to_string(),
        };

        query_client.balance(request).await?.into_inner().try_into()
    }

    /// Gets all coin balances of the specified address with optional pagination
    pub async fn all_balances(&mut self, address: &str) -> Result<CoinsResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
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
    pub async fn spendable_balances(&mut self, address: &str) -> Result<CoinsResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
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
    pub async fn bank_params(&mut self) -> Result<bank::QueryParamsResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }

    /// Gets metadata for the specified coin denomination if it exists, errors otherwise
    pub async fn denom_metadata(
        &mut self,
        denom: &str,
    ) -> Result<bank::QueryDenomMetadataResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomMetadataRequest {
            denom: denom.to_string(),
        };

        Ok(query_client.denom_metadata(request).await?.into_inner())
    }

    /// Gets the metadata for all coin denominations defined in the bank module.
    pub async fn all_denoms_metadata(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<bank::QueryDenomsMetadataResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomsMetadataRequest { pagination };

        Ok(query_client.denoms_metadata(request).await?.into_inner())
    }

    /// Gets the supply of the specified coin denomination
    pub async fn supply_of(&mut self, denom: &str) -> Result<Coin> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
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
    pub async fn total_supply(&mut self, pagination: Option<PageRequest>) -> Result<CoinsResponse> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
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
