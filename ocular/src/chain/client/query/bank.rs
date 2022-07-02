//! Queries for the [Bank module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/bank/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`BankQueryClient`] directly.
use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use tonic::transport::Channel;

use crate::{
    cosmos_modules::bank,
    error::{ChainClientError, GrpcError},
};

use super::{ChainClient, QueryClient};

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
        address: String,
    ) -> Result<Vec<Coin>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryAllBalancesRequest {
            address,
            pagination: None,
        };
        let response = query_client
            .all_balances(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.balances)
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

    pub async fn query_denoms_metadata(&mut self) -> Result<Vec<bank::Metadata>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryDenomsMetadataRequest { pagination: None };
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
            Some(a) => Ok(a),
            None => Err(ChainClientError::ModuleQuery(format!(
                "empty result. denom {} is probably invalid!",
                denom
            ))),
        };
    }

    pub async fn query_total_supply(&mut self) -> Result<Vec<Coin>, ChainClientError> {
        let mut query_client = self.get_query_client::<BankQueryClient>().await?;
        let request = bank::QueryTotalSupplyRequest { pagination: None };
        let response = query_client
            .total_supply(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.supply)
    }
}
