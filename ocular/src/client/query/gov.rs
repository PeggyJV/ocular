//! Queries for the [Gov module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/gov/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`GovQueryClient`] directly.
use tonic::transport::Channel;

use crate::cosmos_modules::gov;

use super::GrpcClient;

/// The gov module's query client proto definition
pub type GovQueryClient = gov::query_client::QueryClient<Channel>;

impl GrpcClient for GovQueryClient {}
