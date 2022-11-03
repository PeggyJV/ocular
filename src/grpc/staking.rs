//! Queries and messages for the [Staking module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/staking/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`StakingQueryClient`] directly.
use std::str::FromStr;

use async_trait::async_trait;
use cosmrs::{staking::*, tx::Msg, AccountId, Any, Denom};
use eyre::{Context, Report, Result};
use tonic::transport::Channel;

use crate::{
    cosmrs::proto::cosmos::staking::v1beta1 as staking,
    tx::{ModuleMsg, UnsignedTx},
};

use super::{ConstructClient, GrpcClient, PageRequest};

/// The staking module's query client proto definition
pub type StakingQueryClient = staking::query_client::QueryClient<Channel>;

#[async_trait]
impl ConstructClient<StakingQueryClient> for StakingQueryClient {
    async fn new_client(endpoint: String) -> Result<Self> {
        StakingQueryClient::connect(endpoint.to_owned())
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl GrpcClient {
    /// Params queries the parameters of slashing module
    pub async fn query_staking_params(&mut self) -> Result<staking::QueryParamsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryParamsRequest {};

        Ok(query_client.params(request).await?.into_inner())
    }
    /// Validator queries validator info for given validator address.
    pub async fn query_validator(
        &mut self,
        validator_address: &str,
    ) -> Result<staking::QueryValidatorResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryValidatorRequest {
            validator_addr: validator_address.to_string(),
        };

        Ok(query_client.validator(request).await?.into_inner())
    }

    /// Validators queries all validators that match the given status.
    pub async fn query_validators(
        &mut self,
        status: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryValidatorsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryValidatorsRequest {
            status: status.to_string(),
            pagination,
        };

        Ok(query_client.validators(request).await?.into_inner())
    }

    /// ValidatorDelegations queries delegate info for given validator.
    pub async fn query_validator_delegations(
        &mut self,
        validator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryValidatorDelegationsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_validator_unbonding_delegations(
        &mut self,
        validator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryValidatorUnbondingDelegationsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_delegation(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<staking::QueryDelegationResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryDelegationRequest {
            delegator_addr: delegator_address.to_string(),
            validator_addr: validator_address.to_string(),
        };

        Ok(query_client.delegation(request).await?.into_inner())
    }

    /// UnbondingDelegation queries unbonding info for given validator delegator pair.
    pub async fn query_unbonding_delegation(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<staking::QueryUnbondingDelegationResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_delegator_delegations(
        &mut self,
        delegator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryDelegatorDelegationsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_delegator_unbonding_delegations(
        &mut self,
        delegator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryDelegatorUnbondingDelegationsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_redelegations(
        &mut self,
        delegator_address: &str,
        src_validator_address: &str,
        dst_validator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryRedelegationsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryRedelegationsRequest {
            delegator_addr: delegator_address.to_string(),
            src_validator_addr: src_validator_address.to_string(),
            dst_validator_addr: dst_validator_address.to_string(),
            pagination,
        };

        Ok(query_client.redelegations(request).await?.into_inner())
    }

    /// DelegatorValidator queries validator info for given delegator validator pair.
    pub async fn query_delegator_validator(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Result<staking::QueryDelegatorValidatorResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_delegator_validators(
        &mut self,
        delegator_address: &str,
        pagination: Option<PageRequest>,
    ) -> Result<staking::QueryDelegatorValidatorsResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
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
    pub async fn query_historical_info(
        &mut self,
        height: u64,
    ) -> Result<staking::QueryHistoricalInfoResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryHistoricalInfoRequest {
            height: height as i64,
        };

        Ok(query_client.historical_info(request).await?.into_inner())
    }

    /// Pool queries the pool info.
    pub async fn query_pool(&mut self) -> Result<staking::QueryPoolResponse> {
        let query_client = self.get_client::<StakingQueryClient>().await?;
        let request = staking::QueryPoolRequest {};

        Ok(query_client.pool(request).await?.into_inner())
    }
}

/// Represents a [Staking module message](https://docs.cosmos.network/v0.45/modules/staking/03_messages.html)
///
/// MsgCreateValidator and MsgEditValidator are currently unimplimented
pub enum Staking<'m> {
    /// Delegate stake to a validator. Represents a [`MsgDelegate`]
    Delegate {
        /// Address of the delegating account
        delegator_address: &'m str,
        /// Address of the validator being delegated to
        validator_address: &'m str,
        /// Delegation amount
        amount: u128,
        /// Delegation coin denomination
        denom: &'m str,
    },
    /// Undelegate (remove) stake from a validator. Represents a [`MsgUndelegate`]
    Undelegate {
        /// Address of the undelegating account
        delegator_address: &'m str,
        /// Address of the validator being undelegated from
        validator_address: &'m str,
        /// Amount to undelegate
        amount: u128,
        /// Denomination of the coin to undelegate
        denom: &'m str,
    },
    /// Start a redelegation from one validator to another. Represents a [`MsgBeginRedelegate`]
    BeginRedelegate {
        /// Address of the redelegating account
        delegator_address: &'m str,
        /// Address of the validator delegated stake will be undelegated from
        validator_src_address: &'m str,
        /// Address of the validator the stake will be redelegated to
        validator_dst_address: &'m str,
        /// Redelegation amount
        amount: u128,
        /// Redelegation coin denomination
        denom: &'m str,
    },
}

impl ModuleMsg for Staking<'_> {
    type Error = Report;

    /// Converts the enum into an [`Any`] for use in a transaction
    fn into_any(self) -> Result<Any> {
        match self {
            Staking::Delegate {
                delegator_address,
                validator_address,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };

                MsgDelegate {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_address: AccountId::from_str(validator_address)?,
                    amount,
                }
                .to_any()
            }
            Staking::Undelegate {
                delegator_address,
                validator_address,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };

                MsgUndelegate {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_address: AccountId::from_str(validator_address)?,
                    amount,
                }
                .to_any()
            }
            Staking::BeginRedelegate {
                delegator_address,
                validator_src_address,
                validator_dst_address,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount,
                    denom: Denom::from_str(denom)?,
                };

                MsgBeginRedelegate {
                    delegator_address: AccountId::from_str(delegator_address)?,
                    validator_src_address: AccountId::from_str(validator_src_address)?,
                    validator_dst_address: AccountId::from_str(validator_dst_address)?,
                    amount,
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
        Staking::Delegate {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Staking::Undelegate {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();

        Staking::BeginRedelegate {
            delegator_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_src_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            validator_dst_address: "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u",
            amount: 0,
            denom: "uatom",
        }
        .into_tx()
        .unwrap();
    }
}
