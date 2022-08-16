//! Queries for the [Bank module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/bank/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`BankQueryClient`] directly.
use eyre::{eyre, Result};
use tonic::transport::Channel;

use crate::{
    cosmos_modules::bank,
    Coin,
};

use super::{QueryClient, PageRequest, GrpcClient};

/// The bank module's query client proto definition
pub type BankQueryClient = bank::query_client::QueryClient<Channel>;

impl GrpcClient for BankQueryClient {}

impl QueryClient {
    /// Gets all coin balances of the specified address with optional pagination
    pub async fn all_balances(
        &mut self,
        address: &str,
    ) -> Result<Vec<Coin>> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryAllBalancesRequest {
            address: address.to_string(),
            pagination: None,
        };
        let response = query_client
            .all_balances(request)
            .await?
            .into_inner();
        let mut balances = Vec::<Coin>::new();

        for b in response.balances {
            balances.push(b.try_into()?)
        }

        Ok(balances)
    }

    /// Gets the bank module's params
    pub async fn bank_params(&mut self) -> Result<Option<bank::Params>> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryParamsRequest {};
        let response = query_client
            .params(request)
            .await?
            .into_inner();

        Ok(response.params)
    }

    /// Gets metadata for the specified coin denomination if it exists, errors otherwise
    pub async fn denom_metadata(
        &mut self,
        denom: &str,
    ) -> Result<bank::Metadata> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomMetadataRequest {
            denom: denom.to_string(),
        };
        let response = query_client
            .denom_metadata(request)
            .await?
            .into_inner();
        return match response.metadata {
            Some(md) => Ok(md),
            None => Err(eyre!("Empty denom metadata result. Denom {denom} is probably invalid!")),
        };
    }

    /// Gets the metadata for all coin denominations defined in the bank module.
    pub async fn all_denoms_metadata(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<Vec<bank::Metadata>> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomsMetadataRequest { pagination };
        let response = query_client
            .denoms_metadata(request)
            .await?
            .into_inner();

        Ok(response.metadatas)
    }

    /// Gets the supply of the specified coin denomination
    pub async fn supply(&mut self, denom: &str) -> Result<Coin> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QuerySupplyOfRequest {
            denom: denom.to_string(),
        };
        let response = query_client
            .supply_of(request)
            .await?
            .into_inner();
        return match response.amount {
            Some(a) => Ok(a.try_into()?),
            None => Err(eyre!("Empty denom supply result. Denom {denom} is probably invalid!")),
        };
    }

    /// Gets the supply of all coin denominations with optional pagination
    pub async fn total_supply(
        &mut self,
        pagination: Option<PageRequest>,
    ) -> Result<Vec<Coin>> {
        let query_client = self.get_grpc_query_client::<BankQueryClient>().await?;
        let request = bank::QueryTotalSupplyRequest { pagination };
        let response = query_client
            .total_supply(request)
            .await?
            .into_inner();
        let mut supply = Vec::<Coin>::new();

        for s in response.supply {
            supply.push(s.try_into()?)
        }

        Ok(supply)
    }
}
