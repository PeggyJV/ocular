//! Queries for the [Gov module](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/gov/v1beta1/query.proto). If you need a query that does not have a method wrapper here, you can use the [`GovQueryClient`] directly.
use async_trait::async_trait;
use eyre::{Context, Result};
use tonic::transport::Channel;

use crate::cosmrs::proto::cosmos::gov::v1beta1::{self as gov, QueryProposalsResponse, QueryVotesResponse};

use super::{GrpcClient, PageRequest, QueryClient};

/// The gov module's query client proto definition
pub type GovQueryClient = gov::query_client::QueryClient<Channel>;

#[async_trait]
impl GrpcClient for GovQueryClient {
    type ClientType = Self;

    async fn make_client(endpoint: String) -> Result<Self::ClientType> {
        GovQueryClient::connect(endpoint)
            .await
            .wrap_err("Failed to make gRPC connection")
    }
}

impl QueryClient {
    /// Params queries all parameters of the gov module.
    pub async fn gov_params(&mut self, params_type: String) -> Result<gov::QueryParamsResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryParamsRequest {params_type};

        Ok(query_client
            .params(request)
            .await?
            .into_inner())
    }

    /// Proposal queries proposal details based on ProposalID.
    pub async fn proposal(&mut self, proposal_id: u64) -> Result<gov::QueryProposalResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryProposalRequest { proposal_id };

        Ok(query_client
            .proposal(request)
            .await?
            .into_inner())
    }

    /// Proposals queries all proposals based on given status.
    pub async fn proposals(
        &mut self,
        proposal_status: i32,
        voter: String,
        depositor: String,
        pagination: Option<PageRequest>,
    ) -> Result<QueryProposalsResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryProposalsRequest {
            proposal_status,
            voter,
            depositor,
            pagination
        };

        Ok(query_client
            .proposals(request)
            .await?
            .into_inner())
    }

    /// Vote queries voted information based on proposalID, voterAddr.
    pub async fn vote(&mut self, proposal_id: u64, voter: String) -> Result<gov::QueryVoteResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryVoteRequest {
            proposal_id,
            voter,
        };

        Ok(query_client
            .vote(request)
            .await?
            .into_inner())
    }

    /// Votes queries votes of a given proposal.
    pub async fn votes(
        &mut self,
        proposal_id: u64,
        pagination: Option<PageRequest>,
    ) -> Result<QueryVotesResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryVotesRequest {
            proposal_id,
            pagination
        };

        Ok(query_client
            .votes(request)
            .await?
            .into_inner())
    }

    /// Deposit queries single deposit information based proposalID, depositAddr.
    pub async fn deposit(&mut self, proposal_id: u64, depositor: String) -> Result<gov::QueryDepositResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryDepositRequest {
            proposal_id,
            depositor
        };

        Ok(query_client
            .deposit(request)
            .await?
            .into_inner())
    }

    /// Deposits queries all deposits of a single proposal.
    pub async fn deposits(
        &mut self,
        proposal_id: u64,
        pagination: Option<PageRequest>,
    ) -> Result<gov::QueryDepositsResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryDepositsRequest {
            proposal_id,
            pagination
        };

        Ok(query_client
            .deposits(request)
            .await?
            .into_inner())
    }

    /// TallyResult queries the tally of a proposal vote.
    pub async fn tally_result(&mut self, proposal_id: u64) -> Result<gov::QueryTallyResultResponse> {
        let query_client = self.get_grpc_query_client::<GovQueryClient>().await?;
        let request = gov::QueryTallyResultRequest {
            proposal_id,
        };

        Ok(query_client
            .tally_result(request)
            .await?
            .into_inner())
    }
}
