//! Queries for the [Slashing module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/slashing/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`SlashingQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::slashing::v1beta1 as slashing;

use super::{GrpcClient, PageRequest, QueryClient};

/// The slashing module's query client proto definition
pub type SlashingQueryClient = slashing::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for SlashingQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        SlashingQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Params queries the parameters of slashing module
    pub async fn slashing_params(&mut self) -> Result<slashing::QueryParamsResponse> {
        let query_client = self.get_grpc_query_client::<SlashingQueryClient>().await?;
        let request = slashing::QueryParamsRequest {};

        Ok(query_client
            .params(request)
            .await?
            .into_inner())
    }


    /// SigningInfo queries the signing info of given cons address
    pub async fn signing_info(&mut self, cons_address: &str) -> Result<slashing::QuerySigningInfoResponse> {
        let query_client = self.get_grpc_query_client::<SlashingQueryClient>().await?;
        let request = slashing::QuerySigningInfoRequest { cons_address: cons_address.to_string() };

        Ok(query_client
            .signing_info(request)
            .await?
            .into_inner())
    }

    /// SigningInfos queries signing info of all validators
    pub async fn signing_infos(&mut self, pagination: Option<PageRequest>) -> Result<slashing::QuerySigningInfosResponse> {
        let query_client = self.get_grpc_query_client::<SlashingQueryClient>().await?;
        let request = slashing::QuerySigningInfosRequest {pagination};

        Ok(query_client
            .signing_infos(request)
            .await?
            .into_inner())
    }
}
