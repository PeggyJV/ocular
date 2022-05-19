use crate::{
    cosmos_modules::{base::Coin, *},
    error::{ChainClientError, GrpcError, RpcError},
};
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use prost::Message;
use tendermint_rpc::Client;
use tonic::transport::Channel;

use super::ChainClient;

pub type AuthQueryClient = auth::query_client::QueryClient<Channel>;
pub type BankQueryClient = bank::query_client::QueryClient<Channel>;
pub type DistributionQueryClient = distribution::query_client::QueryClient<Channel>;
pub type EvidenceQueryClient = evidence::query_client::QueryClient<Channel>;
pub type GovQueryClient = gov::query_client::QueryClient<Channel>;
pub type MintQueryClient = mint::query_client::QueryClient<Channel>;
pub type ParamsQueryClient = params::query_client::QueryClient<Channel>;
pub type SlashingQueryClient = slashing::query_client::QueryClient<Channel>;
pub type StakingQueryClient = staking::query_client::QueryClient<Channel>;

impl ChainClient {
    // Auth queries
    pub async fn get_auth_query_client(&self) -> Result<AuthQueryClient, ChainClientError> {
        AuthQueryClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(|e| GrpcError::Connection(e).into())
    }

    pub async fn query_account(
        &self,
        address: String,
    ) -> Result<auth::BaseAccount, ChainClientError> {
        let mut query_client = self.get_auth_query_client().await?;
        let request = auth::QueryAccountRequest { address };
        let response = query_client
            .account(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        let any = response.account.unwrap();

        Ok(auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
    }

    pub async fn query_accounts(
        &self,
        pagination: Option<PageRequest>,
    ) -> Result<Vec<auth::BaseAccount>, ChainClientError> {
        let mut query_client = self.get_auth_query_client().await?;
        let request = auth::QueryAccountsRequest { pagination };

        Ok(query_client
            .accounts(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner()
            .accounts
            .iter()
            .map(|any| auth::BaseAccount::decode(&any.value as &[u8]).unwrap())
            .collect())
    }

    // Bank queries
    pub async fn get_bank_query_client(&self) -> Result<BankQueryClient, ChainClientError> {
        BankQueryClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(|e| GrpcError::Connection(e).into())
    }

    pub async fn query_all_balances(&self, address: String) -> Result<Vec<Coin>, ChainClientError> {
        let mut query_client = self.get_bank_query_client().await?;
        let request = bank::QueryAllBalancesRequest {
            address,
            pagination: None,
        };
        let response = query_client
            .all_balances(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.balances)
    }

    pub async fn query_bank_params(&self) -> Result<Option<bank::Params>, ChainClientError> {
        let mut query_client = self.get_bank_query_client().await?;
        let request = bank::QueryParamsRequest {};
        let response = query_client
            .params(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.params)
    }

    pub async fn query_denom_metadata(
        &self,
        denom: &str,
    ) -> Result<bank::Metadata, ChainClientError> {
        let mut query_client = self.get_bank_query_client().await?;
        let request = bank::QueryDenomMetadataRequest {
            denom: denom.to_string(),
        };
        let response = query_client
            .denom_metadata(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        return match response.metadata {
            Some(md) => Ok(md),
            None => Err(ChainClientError::ModuleQuery(format!(
                "empty result. denom {} is probably invalid!",
                denom
            ))),
        };
    }

    pub async fn query_denoms_metadata(&self) -> Result<Vec<bank::Metadata>, ChainClientError> {
        let mut query_client = self.get_bank_query_client().await?;
        let request = bank::QueryDenomsMetadataRequest { pagination: None };
        let response = query_client
            .denoms_metadata(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.metadatas)
    }

    pub async fn query_supply(&self, denom: &str) -> Result<Coin, ChainClientError> {
        let mut query_client = self.get_bank_query_client().await?;
        let request = bank::QuerySupplyOfRequest {
            denom: denom.to_string(),
        };
        let response = query_client
            .supply_of(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();
        return match response.amount {
            Some(a) => Ok(a),
            None => Err(ChainClientError::ModuleQuery(format!(
                "empty result. denom {} is probably invalid!",
                denom
            ))),
        };
    }

    pub async fn query_total_supply(&self) -> Result<Vec<Coin>, ChainClientError> {
        let mut query_client = self.get_bank_query_client().await?;
        let request = bank::QueryTotalSupplyRequest { pagination: None };
        let response = query_client
            .total_supply(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response.supply)
    }

    // RPC queries
    pub async fn query_latest_height(&self) -> Result<u64, ChainClientError> {
        let status = self
            .rpc_client
            .status()
            .await
            .map_err(RpcError::TendermintStatus)?;
        Ok(status.sync_info.latest_block_height.value())
    }
}
