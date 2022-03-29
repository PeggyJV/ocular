use crate::{
    chain::client::tx::{Account, TxMetadata},
    cosmos_modules::{base::Coin, *},
    error::{ChainClientError, GrpcError, TxError},
};
use cosmos_sdk_proto::cosmos::authz::v1beta1::{GenericAuthorization, MsgExec, MsgGrant};
use k256::elliptic_curve::group::GroupEncoding;
use prost::Message;
use tendermint_rpc::Client;
use tonic::transport::Channel;

use cosmrs::proto::cosmos::bank::v1beta1::MsgSend;

use super::ChainClient;

pub type AuthzMessageClient = authz::msg_client::MsgClient<Channel>;

impl ChainClient {
    // Authz messages
    pub async fn get_authz_msg_client(&self) -> Result<AuthzMessageClient, ChainClientError> {
        self.check_for_grpc_address()?;

        AuthzMessageClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(|e| GrpcError::Connection(e).into())
    }

    // Grant Authorization
    // TODO: support other types of authorization grants other than GenericAuthorization for send messages.
    pub async fn grant_authorization(
        &self,
        granter: Account,
        grantee: Account,
        timestamp: Option<prost_types::Timestamp>,
    ) -> Result<(), ChainClientError> {
        let mut msg_client = self.get_authz_msg_client().await?;

        let request = MsgGrant {
            granter: granter.id.as_ref().to_string(),
            grantee: grantee.id.as_ref().to_string(),
            grant: Some(authz::Grant {
                authorization: Some(prost_types::Any {
                    type_url: String::from("/cosmos.authz.v1beta1.GenericAuthorization"),
                    value: GenericAuthorization {
                        msg: String::from("/cosmos.bank.v1beta1.MsgSend"),
                    }
                    .encode_to_vec(),
                }),
                expiration: timestamp,
            }),
        };

        msg_client.grant(request);

        Ok(())
    }
}

// Disclaimer on testing: Since the above commands inherently require chains to operate, testing is primarily deferred to integration tests in ocular/tests/single_node_chain_txs.rs

#[cfg(test)]
mod tests {
    use crate::chain::{self, client::ChainClient};
    use assay::assay;

    #[assay]
    async fn gets_authz_client() {
        let client = ChainClient::new(chain::COSMOSHUB).unwrap();

        client
            .get_authz_msg_client()
            .await
            .expect("failed to get authz msg client");
    }
}
