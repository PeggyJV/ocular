use assay::assay;
use ocular::{
    client::ChainClient,
};

#[assay]
async fn chain_client_construction() {
    ChainClient::new(
        "https://cosmoshub.strange.love:9090",
        "https://cosmoshub-4.technofractal.com:443",
    )
    .unwrap();
}

// the rpc endpoints are unreliable so only run this when explicity requested
#[cfg(skip)]
#[assay]
async fn query_latest_block_height() {
    let client = ChainClient::create(chain::COSMOSHUB).expect("failed to get test client");
    dbg!("rpc address:", &client.config.rpc_address);
    client
        .query_latest_height()
        .await
        .expect("failed to query latest height");
}

#[cfg(skip)]
#[assay]
async fn auth_queries() {
    let client = ChainClient::create(chain::COSMOSHUB).unwrap();

    client
        .query_account("cosmos1j5f60735tg604tjd0ts7z22hsmva6nznz8na6q".to_string())
        .await
        .expect("failed to query account");
    let pagination = PageRequest {
        key: Vec::<u8>::default(),
        offset: 1,
        limit: 1,
        count_total: false,
        reverse: false,
    };
    client
        .query_accounts(Some(pagination))
        .await
        .expect("failed to query account");
}

#[cfg(skip)]
#[assay]
async fn bank_queries() {
    let client = ChainClient::create(chain::COSMOSHUB).expect("failed to get test client");

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
