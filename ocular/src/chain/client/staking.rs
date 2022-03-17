use crate::{
    cosmos_modules::*,
    error::{ChainClientError, GrpcError},
};

use tonic::transport::Channel;

use super::ChainClient;

pub type StakingMsgClient = staking::msg_client::MsgClient<Channel>;

impl ChainClient {
    /// Get Staking Msg Client
    pub async fn get_staking_msg_client(&self) -> Result<StakingMsgClient, GrpcError> {
        self.check_for_grpc_address()?;

        StakingMsgClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(GrpcError::Connection)
    }
}

// ---------------------------------- Tests ----------------------------------
// Super basic unit tests since these operations inherently require chains
#[cfg(test)]
mod tests {
    use crate::chain::client::ChainClient;
    use assay::assay;

    #[assay]
    async fn staking_msg_client_initialization() {
        let client = ChainClient::new("cosmoshub").unwrap();

        client
            .get_staking_msg_client()
            .await
            .expect("failed to get staking msg client");
    }
}
