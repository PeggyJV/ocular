//! Queries and messages for the [Mint module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/mint/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`MintQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::mint::v1beta1 as mint;

use super::{ConstructClient, GrpcClient};

/// The mint module's query client proto definition
pub type MintQueryClient = mint::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<MintQueryClient> for MintQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        MintQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Params queries all parameters of the mint module.
    pub async fn query_mint_params(&mut self) -> Result<mint::QueryParamsResponse> {
        let query_client = self.get_client::<MintQueryClient>().await?;
        let request = mint::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }

    /// Inflation returns the current minting inflation value.
    pub async fn query_inflation(&mut self) -> Result<mint::QueryInflationResponse> {
        let query_client = self.get_client::<MintQueryClient>().await?;
        let request = mint::QueryInflationRequest {};

        Ok(query_client.inflation(request).await?.into_inner())
    }

    /// AnnualProvisions returns current minting annual provisions value.
    pub async fn query_annual_provisions(&mut self) -> Result<mint::QueryAnnualProvisionsResponse> {
        let query_client = self.get_client::<MintQueryClient>().await?;
        let request = mint::QueryAnnualProvisionsRequest {};

        Ok(query_client.annual_provisions(request).await?.into_inner())
    }
}
