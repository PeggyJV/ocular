//! Queries for the [Evidence module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/evidence/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`EvidenceQueryClient`] directly.
use tonic::transport::Channel;

use crate::cosmos_modules::evidence;

use super::GrpcClient;

/// The evidence module's query client proto definition
pub type EvidenceQueryClient = evidence::query_client::QueryClient<Channel>;

impl GrpcClient for EvidenceQueryClient {}
