//! Queries for the [Mint module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/mint/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`MintQueryClient`] directly.
use tonic::transport::Channel;

use crate::cosmos_modules::mint;

use super::GrpcClient;

/// The mint module's query client proto definition
pub type MintQueryClient = mint::query_client::QueryClient<Channel>;

impl GrpcClient for MintQueryClient {}
