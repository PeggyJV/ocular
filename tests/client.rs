#![cfg(feature = "query")]
use crate::common::*;

use assay::assay;
use ocular::query::{AuthQueryClient, BankQueryClient, PageRequest};

mod common;

#[assay]
async fn chain_client_construction() {
    new_cosmos_client();
}

#[assay]
async fn auth_queries() {
    let mut client = new_cosmos_client();

    client
        .account(HUB_TEST_ADDRESS)
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

    assert!(!denoms_metadata.metadatas.is_empty());
    assert!(!total_supply.balances.is_empty());
}

#[assay]
async fn grpc_pool() {
    let mut qclient = new_cosmos_client();

    qclient
        .account(HUB_TEST_ADDRESS)
        .await
        .expect("failed to query account");
    let _balance = qclient
        .balance(HUB_TEST_ADDRESS, "uatom")
        .await
        .expect("failed to query balance");

    assert!(qclient.has_grpc_client::<AuthQueryClient>());
    assert!(qclient.has_grpc_client::<BankQueryClient>());
}
