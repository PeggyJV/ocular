use assay::assay;
use ocular::chain_client::ChainClient;

#[assay]
async fn create_chain_client() {
    let client = ChainClient::new("cosmoshub");

    client.unwrap();
}
