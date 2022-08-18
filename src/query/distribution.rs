//! Queries for the [Distribution module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/distribution/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`DistributionQueryClient`] directly.
use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::{distribution::v1beta1::{ValidatorOutstandingRewards, ValidatorAccumulatedCommission, ValidatorSlashEvent, QueryDelegationTotalRewardsResponse}, base::v1beta1::DecCoin};
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::{cosmos_modules::distribution, QueryClient};

use super::GrpcClient;

/// The distribution module's query client proto definition
pub type DistributionQueryClient = distribution::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for DistributionQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        DistributionQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Gets the distribution module's params
    pub async fn distribution_params(&mut self) -> Result<Option<distribution::Params>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryParamsRequest {};
        let response = query_client.params(request).await?.into_inner();

        Ok(response.params)
    }

    pub async fn validator_outstanding_rewards(&mut self, validator_address: &str) -> Result<Option<ValidatorOutstandingRewards>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryValidatorOutstandingRewardsRequest {
            validator_address: validator_address.to_string(),
        };

        Ok(query_client
            .validator_outstanding_rewards(request)
            .await?
            .into_inner()
            .rewards)
    }

    pub async fn validator_commission(&mut self, validator_address: &str) -> Result<Option<ValidatorAccumulatedCommission>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryValidatorCommissionRequest {
            validator_address: validator_address.to_string(),
        };

        Ok(query_client
            .validator_commission(request)
            .await?
            .into_inner()
            .commission)
    }

    pub async fn validator_slashes(
        &mut self,
        validator_address: &str,
        starting_height: u64,
        ending_height: u64,
    ) -> Result<Vec<ValidatorSlashEvent>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryValidatorSlashesRequest {
            validator_address: validator_address.to_string(),
            starting_height,
            ending_height,
            pagination: None,
        };

        Ok(query_client
            .validator_slashes(request)
            .await?
            .into_inner()
            .slashes)
    }

    pub async fn delegation_rewards(
        &mut self,
        delegate_address: &str,
        validator_address: &str,
    ) -> Result<Vec<DecCoin>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryDelegationRewardsRequest {
            delegator_address: delegate_address.to_string(),
            validator_address: validator_address.to_string(),
        };

        Ok(query_client
            .delegation_rewards(request)
            .await?
            .into_inner()
            .rewards)
    }

    pub async fn delegation_total_rewards(&mut self, delegate_address: &str) -> Result<QueryDelegationTotalRewardsResponse> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryDelegationTotalRewardsRequest {
            delegator_address: delegate_address.to_string(),
        };

        Ok(query_client
            .delegation_total_rewards(request)
            .await?
            .into_inner())
    }


    pub async fn delegator_validators(&mut self, delegate_address: &str) -> Result<Vec<String>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryDelegatorValidatorsRequest {
            delegator_address: delegate_address.to_string(),
        };

        Ok(query_client
            .delegator_validators(request)
            .await?
            .into_inner()
            .validators)
    }

    pub async fn delegator_withdraw_address(&mut self, delegate_address: &str) -> Result<String> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryDelegatorWithdrawAddressRequest {
            delegator_address: delegate_address.to_string(),
        };

        Ok(query_client
            .delegator_withdraw_address(request)
            .await?
            .into_inner()
            .withdraw_address)
    }

    pub async fn community_pool(&mut self, delegate_address: &str) -> Result<Vec<DecCoin>> {
        let query_client = self.get_grpc_query_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryCommunityPoolRequest {};

        Ok(query_client
            .community_pool(request)
            .await?
            .into_inner()
            .pool)
    }
}
