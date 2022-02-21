use assay::assay;
use ocular::{chain_client::ChainClient, chain_info};

#[assay]
async fn create_chain_client() {
    let hub = chain_info::get_cosmoshub_info().await.unwrap();
    let config = hub.get_chain_config().unwrap();
    let client = ChainClient::new(config);

    client.unwrap();
}
