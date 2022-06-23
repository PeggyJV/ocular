use crate::{
    account::AccountInfo,
    cosmos_modules::{
        authz::{self, *},
        feegrant::{BasicAllowance, MsgGrantAllowance},
    },
    error::{ChainClientError, GrpcError, TxError},
    tx::TxMetadata,
};
use cosmrs::{tx, AccountId};
use prost::Message;
use tendermint_rpc::endpoint::broadcast::tx_commit::Response;
use tonic::transport::Channel;

use super::ChainClient;

pub type AuthzQueryClient = authz::query_client::QueryClient<Channel>;

impl ChainClient {
    // Authz queries
    // TODO: Refractor code accross grpc clients by having keystore implement Sync (https://github.com/PeggyJV/ocular/pull/53#discussion_r880698565)
    pub async fn get_authz_query_client(&mut self) -> Result<AuthzQueryClient, ChainClientError> {
        let mut result: Result<AuthzQueryClient, ChainClientError> =
            Err(TxError::Broadcast(String::from("Client connection never attempted.")).into());

        // Get grpc address randomly each time; shuffles on failures
        for _i in 0u8..self.connection_retry_attempts + 1 {
            // Attempt to use last healthy (or manually set) endpoint if it exists (in config)
            let endpoint: String = if !self.config.grpc_address.is_empty() {
                self.config.grpc_address.clone()
            } else {
                // Get a random endpoint from the cache
                match self.get_random_grpc_endpoint().await {
                    Ok(endpt) => endpt,
                    Err(err) => return Err(GrpcError::MissingEndpoint(err.to_string()).into()),
                }
            };

            result = AuthzQueryClient::connect(endpoint.clone())
                .await
                .map_err(|e| GrpcError::Connection(e).into());

            // Return if result is valid client, or increment failure in cache if being used
            if result.is_ok() {
                // Reset consecutive failed connections to 0
                self.cache
                    .as_mut()
                    .unwrap()
                    .grpc_endpoint_cache
                    .add_item(endpoint.clone(), 0)?;

                // Update config to last healthy grpc connection address
                self.config.grpc_address = endpoint.clone();

                break;
            } else if result.is_err() && self.cache.is_some() {
                // Don't bother updating config grpc address if it fails, it'll be overriden upon a successful connection
                let _res = self
                    .cache
                    .as_mut()
                    .unwrap()
                    .grpc_endpoint_cache
                    .increment_failed_connections(endpoint)?;
            }
        }

        result
    }

    // Query for a specific msg grant
    pub async fn query_authz_grant(
        &mut self,
        granter: &str,
        grantee: &str,
        msg_type_url: &str,
    ) -> Result<QueryGrantsResponse, ChainClientError> {
        let mut query_client = self.get_authz_query_client().await?;

        let request = QueryGrantsRequest {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: msg_type_url.to_string(),
            // TODO: Support pagination if use case arises
            pagination: None,
        };

        let response = query_client
            .grants(request)
            .await
            .map_err(GrpcError::Request)?
            .into_inner();

        Ok(response)
    }

    // Grant Authorization
    // TODO: support other types of authorization grants other than GenericAuthorization for send messages.
    pub async fn grant_send_authorization(
        &mut self,
        granter: AccountInfo,
        grantee: AccountId,
        expiration_timestamp: Option<prost_types::Timestamp>,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgGrant {
            granter: granter.id.to_string(),
            grantee: grantee.to_string(),
            grant: Some(Grant {
                authorization: Some(prost_types::Any {
                    type_url: String::from("/cosmos.authz.v1beta1.GenericAuthorization"),
                    value: GenericAuthorization {
                        msg: String::from("/cosmos.bank.v1beta1.MsgSend"),
                    }
                    .encode_to_vec(),
                }),
                expiration: expiration_timestamp,
            }),
        };
        let account = self.query_account(granter.id.as_ref().to_string()).await?;
        let (account_number, sequence) = (account.account_number, account.sequence);

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, account_number, sequence, tx_body, tx_metadata)
            .await
    }

    // Revoke Authorization
    // TODO: support other types of authorization revokes other than send messages.
    pub async fn revoke_send_authorization(
        &mut self,
        granter: AccountInfo,
        grantee: AccountId,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgRevoke {
            granter: granter.id.to_string(),
            grantee: grantee.to_string(),
            msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
        };
        let account = self.query_account(granter.id.as_ref().to_string()).await?;
        let (account_number, sequence) = (account.account_number, account.sequence);

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, account_number, sequence, tx_body, tx_metadata)
            .await
    }

    // Execute a transaction previously authorized by another account on its behalf
    pub async fn execute_authorized_tx(
        &mut self,
        grantee: AccountInfo,
        msgs: Vec<::prost_types::Any>,
        tx_metadata: Option<TxMetadata>,
    ) -> Result<Response, ChainClientError> {
        let msg = MsgExec {
            grantee: grantee.id.to_string(),
            msgs,
        };
        let account = self.query_account(grantee.id.as_ref().to_string()).await?;
        let (account_number, sequence) = (account.account_number, account.sequence);

        // Build tx body.
        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
            value: msg.encode_to_vec(),
        };
        let tx_metadata = match tx_metadata {
            Some(tm) => tm,
            None => self.get_basic_tx_metadata().await?,
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(grantee, account_number, sequence, tx_body, tx_metadata)
            .await
    }

    // Basic fee allowance
    pub async fn perform_basic_allowance_fee_grant(
        &mut self,
        granter: AccountInfo,
        grantee: AccountId,
        expiration: Option<prost_types::Timestamp>,
        // TODO: Standardize below Coin type to common cosmrs coin type once FeeGrants get looped in.
        spend_limit: cosmos_sdk_proto::cosmos::base::v1beta1::Coin,
        tx_metadata: TxMetadata,
    ) -> Result<Response, ChainClientError> {
        let allowance = BasicAllowance {
            spend_limit: vec![spend_limit],
            expiration,
        };
        let msg = MsgGrantAllowance {
            granter: granter.id.to_string(),
            grantee: grantee.to_string(),
            allowance: Some(prost_types::Any {
                type_url: String::from("/cosmos.feegrant.v1beta1.BasicAllowance"),
                value: allowance.encode_to_vec(),
            }),
        };
        let account = self.query_account(granter.id.as_ref().to_string()).await?;
        let (account_number, sequence) = (account.account_number, account.sequence);

        let msg_any = prost_types::Any {
            type_url: String::from("/cosmos.feegrant.v1beta1.MsgGrantAllowance"),
            value: msg.encode_to_vec(),
        };
        let tx_body = tx::Body::new(vec![msg_any], &tx_metadata.memo, tx_metadata.timeout_height);

        self.sign_and_send_msg(granter, account_number, sequence, tx_body, tx_metadata)
            .await
    }
}

// Disclaimer on testing: Since the above commands inherently require chains to operate, testing is primarily deferred to integration tests in ocular/tests/single_node_chain_txs.rs

#[cfg(test)]
mod tests {
    use crate::chain::{self, client::ChainClient};
    use assay::assay;

    #[assay]
    async fn gets_authz_client() {
        let mut client = ChainClient::create(chain::COSMOSHUB).unwrap();
        client.config.grpc_address = "http://cosmoshub.strange.love:9090".to_string();

        client
            .get_authz_query_client()
            .await
            .expect("failed to get authz query client");
    }
}
