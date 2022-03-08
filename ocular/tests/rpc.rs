use assay::assay;
use ocular::chain::{client::ChainClient, Chains};

#[assay]
async fn create_chain_client() {
    let client = ChainClient::new(Chains::CosmosHub);

    client.unwrap();
}

#[assay]
async fn create_unsupported_chain_client() {
    let client = ChainClient::new_unsupported("cosmoshub");

    client.unwrap();
}
