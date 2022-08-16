//! Queries for the [Staking module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/staking/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`StakingQueryClient`] directly.
use tonic::transport::Channel;

use crate::cosmos_modules::staking;

use super::GrpcClient;

/// The staking module's query client proto definition
pub type StakingQueryClient = staking::query_client::QueryClient<Channel>;

impl GrpcClient for StakingQueryClient {}
