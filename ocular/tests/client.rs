use assay::assay;
use ocular::{
    chain::{
        self,
        client::{cache::Cache, ChainClient, ChainClientBuilder},
        config::ChainClientConfig,
        SOMMELIER,
    },
    keyring::Keyring,
};

#[assay]
async fn create_chain_client() {
    ChainClient::create(chain::COSMOSHUB).expect("failed to create client");
}

#[assay]
async fn manual_chain_client_path() {
    let config = ChainClientConfig {
        account_prefix: "cosmos".to_string(),
        chain_id: "cosmoshub-4".to_string(),
        gas_adjustment: 1.0,
        gas_prices: 100_000u64.to_string(),
        grpc_address: "https://cosmoshub.strange.love:9090".to_string(),
        rpc_address: "https://cosmoshub-4.technofractal.com:443".to_string(),
    };
    let keyring = Keyring::try_default_file_store().unwrap();
    let cache = Some(Cache::create_memory_cache(None).unwrap());

    ChainClient::new(config, keyring, cache).expect("failed to create client");
}

#[assay]
async fn chain_client_builder_path() {
    let grpc_endpoint = "whatever";
    let rpc_endpoint = "http://sommelier.strange.love:26657";
    let client = ChainClientBuilder::new(SOMMELIER)
        .with_grpc_endpoint(grpc_endpoint)
        .with_rpc_endpoint(rpc_endpoint)
        .build()
        .await
        .expect("failed to build client");

    assert_eq!(client.config.grpc_address, grpc_endpoint);
    assert_eq!(client.config.rpc_address, rpc_endpoint);
}

#[assay]
async fn query_latest_block_height() {
    let client = ChainClient::create(chain::COSMOSHUB).expect("failed to get test client");

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
