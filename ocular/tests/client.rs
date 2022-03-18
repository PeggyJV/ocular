use assay::assay;
use ocular::chain::client::ChainClient;

#[assay]
async fn create_chain_client() {
    let client = ChainClient::new("cosmoshub");

    client.unwrap();
}

#[assay]
async fn query_latest_block_height() {
    let client = ChainClient::new("cosmoshub").expect("failed to get test client");

    client
        .query_latest_height()
        .await
        .expect("failed to query latest height");
}

#[assay]
async fn bank_queries() {
    let client = ChainClient::new("cosmoshub").expect("failed to get test client");

    // TO-DO need an address for testing balance query. Maybe include test-specific keys?
    let _denom_metadata = client
        .query_denom_metadata("uatom")
        .await
        .expect("failed to query denom metadata");
    let denoms_metadata = client
        .query_denoms_metadata()
        .await
        .expect("failed to query denoms metadata");
    let _params = client
        .query_bank_params()
        .await
        .expect("failed to query bank params");
    let total_supply = client
        .query_total_supply()
        .await
        .expect("failed to query total supply");

    assert!(!denoms_metadata.is_empty());
    assert!(!total_supply.is_empty());
}
