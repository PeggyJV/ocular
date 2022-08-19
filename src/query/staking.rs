//! Queries for the [Staking module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/staking/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`StakingQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::staking::v1beta1 as staking;

use super::{GrpcClient, PageRequest, QueryClient};

/// The staking module's query client proto definition
pub type StakingQueryClient = staking::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for StakingQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        StakingQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Params queries the parameters of slashing module
    pub async fn staking_params(&mut self) -> Result<staking::QueryParamsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }
    /// Validator queries validator info for given validator address.
    pub async fn validator(
        &mut self,
        validator_address: &str,
    ) -> Result<staking::QueryValidatorResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryValidatorRequest {
            validator_addr: validator_address.to_string(),
        };

        Ok(query_client.validator(request).await?.into_inner())
    }

    /// Validators queries all validators that match the given status.
    pub async fn validators(
        &mut self,
        status: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryValidatorsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryValidatorsRequest {
            status: status.to_string(),
            pagination,
        };

        Ok(query_client.validators(request).await?.into_inner())
    }

    /// ValidatorDelegations queries delegate info for given validator.
    pub async fn validator_delegations(
        &mut self,
        validator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryValidatorDelegationsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryValidatorDelegationsRequest {
            validator_addr: validator_address.to_string(),
            pagination,
        };

        Ok(query_client
            .validator_delegations(request)
            .await?
            .into_inner())
    }

    /// ValidatorUnbondingDelegations queries unbonding delegations of a validator.
    pub async fn validator_unbonding_delegations(
        &mut self,
        validator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryValidatorUnbondingDelegationsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryValidatorUnbondingDelegationsRequest {
            validator_addr: validator_address.to_string(),
            pagination,
        };

        Ok(query_client
            .validator_unbonding_delegations(request)
            .await?
            .into_inner())
    }

    /// Delegation queries delegate info for given validator delegator pair.
    pub async fn delegation(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<staking::QueryDelegationResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryDelegationRequest {
            delegator_addr: delegator_address.to_string(),
            validator_addr: validator_address.to_string(),
        };

        Ok(query_client.delegation(request).await?.into_inner())
    }

    /// UnbondingDelegation queries unbonding info for given validator delegator pair.
    pub async fn unbonding_delegation(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<staking::QueryUnbondingDelegationResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryUnbondingDelegationRequest {
            delegator_addr: delegator_address.to_string(),
            validator_addr: validator_address.to_string(),
        };

        Ok(query_client
            .unbonding_delegation(request)
            .await?
            .into_inner())
    }

    /// DelegatorDelegations queries all delegations of a given delegator address.
    pub async fn delegator_delegations(
        &mut self,
        delegator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryDelegatorDelegationsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryDelegatorDelegationsRequest {
            delegator_addr: delegator_address.to_string(),
            pagination,
        };

        Ok(query_client
            .delegator_delegations(request)
            .await?
            .into_inner())
    }

    /// DelegatorUnbondingDelegations queries all unbonding delegations of a given delegator address.
    pub async fn delegator_unbonding_delegations(
        &mut self,
        delegator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryDelegatorUnbondingDelegationsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryDelegatorUnbondingDelegationsRequest {
            delegator_addr: delegator_address.to_string(),
            pagination,
        };

        Ok(query_client
            .delegator_unbonding_delegations(request)
            .await?
            .into_inner())
    }

    /// Redelegations queries redelegations of given address.
    pub async fn redelegations(
        &mut self,
        delegator_address: &str,
        src_validator_address: &str,
        dst_validator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryRedelegationsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryRedelegationsRequest {
            delegator_addr: delegator_address.to_string(),
            src_validator_addr: src_validator_address.to_string(),
            dst_validator_addr: dst_validator_address.to_string(),
            pagination,
        };

        Ok(query_client.redelegations(request).await?.into_inner())
    }

    /// DelegatorValidator queries validator info for given delegator validator pair.
    pub async fn delegator_validator(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<staking::QueryDelegatorValidatorResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryDelegatorValidatorRequest {
            delegator_addr: delegator_address.to_string(),
            validator_addr: validator_address.to_string(),
        };

        Ok(query_client
            .delegator_validator(request)
            .await?
            .into_inner())
    }

    /// DelegatorValidators queries all validators info for given delegator address.
    pub async fn delegator_validators(
        &mut self,
        delegator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryDelegatorValidatorsResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryDelegatorValidatorsRequest {
            delegator_addr: delegator_address.to_string(),
            pagination,
        };

        Ok(query_client
            .delegator_validators(request)
            .await?
            .into_inner())
    }

    /// HistoricalInfo queries the historical info for given height.
    pub async fn historical_info(
        &mut self,
        height: u64,
    ) -> Result<staking::QueryHistoricalInfoResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryHistoricalInfoRequest {
            height: height as i64,
        };

        Ok(query_client.historical_info(request).await?.into_inner())
    }

    /// Pool queries the pool info.
    pub async fn pool(&mut self) -> Result<staking::QueryPoolResponse> {
        let query_client = self.get_grpc_query_client::<StakingQueryClient>().await?;
        let request = staking::QueryPoolRequest {};

        Ok(query_client.pool(request).await?.into_inner())
    }
}
