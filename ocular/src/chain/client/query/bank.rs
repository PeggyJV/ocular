//! Queries for the [Bank module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/bank/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`BankQueryClient`] directly.
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::{
    cosmos_modules::bank,
    error::{ChainClientError, GrpcError}, Coin,
};

use super::{ChainClient, QueryClient, PageRequest};

pub type BankQueryClient = bank::query_client::QueryClient<Channel>;

#[async_trait]
impl QueryClient for BankQueryClient {
    type Transport = Channel;

    async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        Self::connect(endpoint).await
    }
}

impl ChainClient {
    pub async fn query_all_balances(
        &mut self,
        address: &str,
    ) -> Result<Vec<Coin>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryAllBalancesRequest {
            address: address.to_string(),
            pagination: None,
        };
        let response = query_client
            .all_balances(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        let mut balances = Vec::<Coin>::new();

        for b in response.balances {
            balances.push(b.try_into()?)
        }

        Ok(balances)
    }

    pub async fn query_bank_params(&mut self) -> Result<Option<bank::Params>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryParamsRequest {};
        let response = query_client
            .params(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.params)
    }

    pub async fn query_denom_metadata(
        &mut self,
        denom: &str,
    ) -> Result<bank::Metadata, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomMetadataRequest {
            denom: denom.to_string(),
        };
        let response = query_client
            .denom_metadata(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        return match response.metadata {
            Some(md) => Ok(md),
            None => Err(ChainClientError::ModuleQuery(format!(
                "empty result. denom {} is probably invalid!",
                denom
            ))),
        };
    }

    pub async fn query_denoms_metadata(&mut self, pagination: Option<PageRequest>) -> Result<Vec<bank::Metadata>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomsMetadataRequest { pagination };
        let response = query_client
            .denoms_metadata(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.metadatas)
    }

    pub async fn query_supply(&mut self, denom: &str) -> Result<Coin, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QuerySupplyOfRequest {
            denom: denom.to_string(),
        };
        let response = query_client
            .supply_of(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        return match response.amount {
            Some(a) => Ok(a.try_into()?),
            None => Err(ChainClientError::ModuleQuery(format!(
                "empty result. denom {} is probably invalid!",
                denom
            ))),
        };
    }

    pub async fn query_total_supply(&mut self, pagination: Option<PageRequest>) -> Result<Vec<Coin>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryTotalSupplyRequest { pagination };
        let response = query_client
            .total_supply(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        let mut supply = Vec::<Coin>::new();

        for s in response.supply {
            supply.push(s.try_into()?)
        }

        Ok(supply)
    }
}
