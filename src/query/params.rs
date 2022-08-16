//! Queries for the [Params module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/params/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`ParamsQueryClient`] directly.
use tonic::transport::Channel;

use crate::cosmos_modules::params;

use super::GrpcClient;

/// The params module's query client proto definition
pub type ParamsQueryClient = params::query_client::QueryClient<Channel>;

impl GrpcClient for ParamsQueryClient {}
