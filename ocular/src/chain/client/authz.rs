use crate::{
    cosmos_modules::{base::Coin, *},
    chain::client::{tx::{Account, TxMetadata},},
    error::{ChainClientError, GrpcError, TxError},
};
use tendermint_rpc::Client;
use tonic::transport::Channel;
use cosmos_sdk_proto::cosmos::authz::v1beta1::{MsgExec, MsgGrant};

use super::ChainClient;

pub type AuthzMessageCleint = authz::msg_client::MsgClient<Channel>;


impl ChainClient {
    // Authz messagees
    pub async fn get_authz_msg_client(&self) -> Result<AuthzMessageCleint, ChainClientError> {
        self.check_for_grpc_address()?;

        AuthzMessageCleint::connect(self.config.grpc_address.clone())
            .await
            .map_err(|e| GrpcError::Connection(e).into())
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
