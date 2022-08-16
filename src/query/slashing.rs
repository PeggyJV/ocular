//! Queries for the [Slashing module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/slashing/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`SlashingQueryClient`] directly.
use tonic::transport::Channel;

use crate::cosmos_modules::slashing;

use super::GrpcClient;

/// The slashing module's query client proto definition
pub type SlashingQueryClient = slashing::query_client::QueryClient<Channel>;

impl GrpcClient for SlashingQueryClient {}
