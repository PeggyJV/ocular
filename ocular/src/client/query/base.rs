//! Query methods for the [Auth module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/auth/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`BaseQueryClient`] directly.
use cosmos_sdk_proto::cosmos::base;
use tonic::transport::Channel;

use super::{ChainClient, QueryClient};

/// The base module's query client proto definition
pub type BaseQueryClient = base::query_client::QueryClient<Channel>;

impl QueryClient for BaseQueryClient {
    type Transport = Channel;
}
