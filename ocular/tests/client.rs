use assay::assay;
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use ocular::chain::{self, client::ChainClient};

#[assay]
async fn create_chain_client() {
    let client = ChainClient::new(chain::COSMOSHUB);

    client.unwrap();
}

#[assay]
async fn query_latest_block_height() {
    let client = ChainClient::new(chain::COSMOSHUB).expect("failed to get test client");

    client
        .query_latest_height()
        .await
        .expect("failed to query latest height");
}

#[cfg(skip)]
#[assay]
async fn auth_queries() {
    let client = ChainClient::new(chain::COSMOSHUB).unwrap();

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
    let client = ChainClient::new(chain::COSMOSHUB).expect("failed to get test client");

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
