//! Queries and messages for the [Distribution module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/distribution/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`DistributionQueryClient`] directly.
use std::str::FromStr;

use crate::{
    cosmrs::proto::cosmos::{
        base::v1beta1::DecCoin,
        distribution::v1beta1::{
            self as distribution, QueryDelegationTotalRewardsResponse,
            ValidatorAccumulatedCommission, ValidatorOutstandingRewards, ValidatorSlashEvent,
        },
    },
    tx::{ModuleMsg, UnsignedTx},
};
use async_trait::async_trait;
use cosmrs::{distribution::*, tx::Msg, AccountId, Any, Denom};
use eyre::{Context, Report, Result};
use tonic::transport::Channel;

use super::{ConstructClient, GrpcClient};

/// The distribution module's query client proto definition
pub type DistributionQueryClient = distribution::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<DistributionQueryClient> for DistributionQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        DistributionQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Gets the distribution module's params
    pub async fn query_distribution_params(&mut self) -> Result<Option<distribution::Params>> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryParamsRequest {};
        let response = query_client.params(request).await?.into_inner();

        Ok(response.params)
    }

    /// Gets a validator's outstanding (unwithdrawn) rewards
    pub async fn query_validator_outstanding_rewards(
        &mut self,
        validator_address: &str,
    ) -> Result<Option<ValidatorOutstandingRewards>> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryValidatorOutstandingRewardsRequest {
            validator_address: validator_address.to_string(),
        };

        Ok(query_client
            .validator_outstanding_rewards(request)
            .await?
            .into_inner()
            .rewards)
    }

    /// Get's a validators outstanding commission earned from delegator's staking rewards
    pub async fn query_validator_commission(
        &mut self,
        validator_address: &str,
    ) -> Result<Option<ValidatorAccumulatedCommission>> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryValidatorCommissionRequest {
            validator_address: validator_address.to_string(),
        };

        Ok(query_client
            .validator_commission(request)
            .await?
            .into_inner()
            .commission)
    }

    /// Gets recorded slashes for a validator
    pub async fn query_validator_slashes(
        &mut self,
        validator_address: &str,
        starting_height: u64,
        ending_height: u64,
    ) -> Result<Vec<ValidatorSlashEvent>> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
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

    /// Gets the outstanding staking rewards of a delegator with a given validator
    pub async fn query_delegation_rewards(
        &mut self,
        delegate_address: &str,
        validator_address: &str,
    ) -> Result<Vec<DecCoin>> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
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

    /// Gets the total outstanding staking rewards of a delegator with all validators
    pub async fn query_delegation_total_rewards(
        &mut self,
        delegate_address: &str,
    ) -> Result<QueryDelegationTotalRewardsResponse> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryDelegationTotalRewardsRequest {
            delegator_address: delegate_address.to_string(),
        };

        Ok(query_client
            .delegation_total_rewards(request)
            .await?
            .into_inner())
    }

    /// Gets a delegator's reward withdraw address
    pub async fn query_delegator_withdraw_address(
        &mut self,
        delegate_address: &str,
    ) -> Result<distribution::QueryDelegatorWithdrawAddressResponse> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryDelegatorWithdrawAddressRequest {
            delegator_address: delegate_address.to_string(),
        };

        Ok(query_client
            .delegator_withdraw_address(request)
            .await?
            .into_inner())
    }

    /// Gets Community Pool funds
    pub async fn query_community_pool(&mut self) -> Result<Vec<DecCoin>> {
        let query_client = self.get_client::<DistributionQueryClient>().await?;
        let request = distribution::QueryCommunityPoolRequest {};

        Ok(query_client
            .community_pool(request)
            .await?
            .into_inner()
            .pool)
    }
}

/// Represents a [Distribution module message](https://docs.cosmos.network/v0.45/modules/distribution/04_messages.html)
pub enum Distribution<'m> {
    /// Set the withdraw address for a delegators rewards. To learn more, see
    /// [MsgSetWithdrawAddress](https://docs.cosmos.network/master/modules/distribution/04_messages.html#msgsetwithdrawaddress).
    /// Represents a [`MsgSetWithdrawAddress`]
    SetWithdrawAddress {
        /// Address of the delegator account to set withdraw address for
        delegator_address: &'m str,
        /// Address delegator rewards should be withdraw to
        withdraw_address: &'m str,
    },
    /// Withdraw delegator rewards. Represents a [`MsgWithdrawDelegatorReward`]
    WithdrawDelegatorReward {
        /// Address of the delegator account
        delegator_address: &'m str,
        /// Address of the validator staking rewards should be withdraw from
        validator_address: &'m str,
    },
    /// Withdraw validator commission earned from delegators' rewards. Represents a [`MsgWithdrawValidatorCommission`]
    WithdrawValidatorCommission {
        /// Address of the validator
        validator_address: &'m str,
    },
    /// Deposit funds to the community pool. Represents a [`MsgFundCommunityPool`]
    FundCommunityPool {
        /// Deposit amount
        amount: u128,
        /// Deposit coin denomination
        denom: &'m str,
        /// Address of the depositing account
        depositor: &'m str,
    },
}

impl ModuleMsg for Distribution<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Distribution::SetWithdrawAddress {
                delegator_address,
                withdraw_address,
            } => MsgSetWithdrawAddress {
                delegator_address: AccountId::from_str(delegator_address)?,
                withdraw_address: AccountId::from_str(withdraw_address)?,
            }
            .to_any(),
            Distribution::WithdrawDelegatorReward {
                delegator_address,
                validator_address,
            } => MsgWithdrawDelegatorReward {
                delegator_address: AccountId::from_str(delegator_address)?,
                validator_address: AccountId::from_str(validator_address)?,
            }
            .to_any(),
            Distribution::WithdrawValidatorCommission { validator_address } => {
                MsgWithdrawValidatorCommission {
                    validator_address: AccountId::from_str(validator_address)?,
                }
                .to_any()
            }
            Distribution::FundCommunityPool {
                amount,
                denom,
                depositor,
            } => {
                let amount = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };
                MsgFundCommunityPool {
                    depositor: AccountId::from_str(depositor)?,
                    amount: vec![amount],
                }
                .to_any()
            }
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
        Distribution::SetWithdrawAddress {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            withdraw_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();

        Distribution::WithdrawDelegatorReward {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();

        Distribution::WithdrawValidatorCommission {
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();

        Distribution::FundCommunityPool {
            amount: 0,
            denom: "uatom",
            depositor: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
        }
        .into_tx()
        .unwrap();
    }
}
