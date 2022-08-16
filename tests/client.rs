use crate::common::*;

use assay::assay;
use ocular::query::{PageRequest, new_grpc_query_client, AuthQueryClient};

mod common;

#[assay]
async fn chain_client_construction() {
    new_cosmos_client();
}

// the rpc endpoints are unreliable so only run this when explicity requested
#[assay]
async fn query_latest_block_height() {
    let client = new_cosmos_client();

    client
        .latest_height()
        .await
        .expect("failed to query latest height");
}

#[assay]
async fn connect_grpc_query_client() {
    new_grpc_query_client::<AuthQueryClient>("http://cosmoshub.strange.love:9090")
        .await
        .expect("failed to connect grpc client");
}

#[assay]
async fn auth_queries() {
    let mut client = new_cosmos_client();

    client
        .account("cosmos1j5f60735tg604tjd0ts7z22hsmva6nznz8na6q")
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
        .all_accounts(Some(pagination))
        .await
        .expect("failed to query account");
}

#[assay]
async fn bank_queries() {
    let mut qclient = new_cosmos_client();

    // TO-DO need an address for testing balance query. Maybe include test-specific keys?
    let _balance = qclient
        .balance(HUB_TEST_ADDRESS, "uatom")
        .await
        .expect("failed to query balance");
    let _balances = qclient
        .all_balances(HUB_TEST_ADDRESS)
        .await
        .expect("failed to query denoms metadata");
    let _denom_metadata = qclient
        .denom_metadata("uatom")
        .await
        .expect("failed to query denom metadata");
    let denoms_metadata = qclient
        .all_denoms_metadata(None)
        .await
        .expect("failed to query denoms metadata");
    let _params = qclient
        .bank_params()
        .await
        .expect("failed to query bank params");
    let total_supply = qclient
        .total_supply(None)
        .await
        .expect("failed to query total supply");

    assert!(!denoms_metadata.is_empty());
    assert!(!total_supply.is_empty());
}
