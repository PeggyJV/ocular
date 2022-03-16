use cosmos_sdk_proto::cosmos::{
    self,
    tx::v1beta1::{
        service_client
    },
};
use crate::error::{TransactionError, ChainClientError};

use cosmrs::{
    tx::{Tx},
};

use tendermint_rpc::Client;
use tonic::transport::Channel;

use super::ChainClient;

pub type TxClient = tx::query_client::QueryClient<Channel>;

impl ChainClient {
    /* 
    pub async fn get_tx_client(&self) -> Result<TxClient, ChainClientError> {
        self.check_for_grpc_address()?;

        AuthQueryClient::connect(self.config.grpc_address.clone())
            .await
            .map_err(|e| GrpcError::Connection(e).into())
    }
    */
    pub async fn send_message(&self, tx: Tx) -> Result<Tx, TransactionError> {
        Err(TransactionError::SerializationError(String::from("aa")))
    }
}