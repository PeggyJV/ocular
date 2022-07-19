use std::{thread, time::Duration};

use cosmrs::{dev, rpc, Tx};
use ocular::{
    account::AccountInfo,
    chain::{
        client::{cache::Cache, ChainClient},
        config::ChainClientConfig,
    },
    keyring::Keyring,
    tx::{Coin, Payment},
};
use rand::Rng;
use rpc::{endpoint::broadcast::tx_commit::Response, HttpClient};

use crate::utils::{
    run_single_node_test, ACCOUNT_PREFIX, CHAIN_ID, DENOM, MULTISEND_BASE_GAS_APPROX,
    PAYMENT_GAS_APPROX, RPC_PORT,
};

mod utils;

#[test]
fn airdrop_single_sender_single_denom() {
    let container_name = "airdrop_test";

    run_single_node_test(container_name, |sender_account: AccountInfo| {
        async move {
            // by brute force, found that ~7400 recipients results in a 1MB transaction
            let recipients = generate_accounts(250);
            let payments = generate_payments(&recipients);
            let total_to_distribute: u64 = payments.iter().map(|p| p.amount).sum();
            let mut chain_client = init_test_chain_client().await;
            let mut txm = chain_client.get_basic_tx_metadata().await.unwrap();
            txm.gas_limit =
                MULTISEND_BASE_GAS_APPROX + (PAYMENT_GAS_APPROX * payments.len() as u64);

            let sender_address = sender_account.address(ACCOUNT_PREFIX).unwrap();
            let sender_starting_balance: u64 = chain_client
                .query_all_balances(&sender_address)
                .await
                .unwrap()[0]
                .amount
                .parse()
                .unwrap();

            println!("Sender starting balance: {}", sender_starting_balance);

            let response = chain_client
                .execute_airdrop(&sender_account, payments.clone(), Some(txm))
                .await
                .unwrap();

            // wait 1 minute for the tx to be included in a block
            wait_for_tx(&chain_client.rpc_client, &response, 10).await;

            let sender_ending_balance: u64 = chain_client
                .query_all_balances(&sender_address)
                .await
                .unwrap()[0]
                .amount
                .parse()
                .unwrap();

            println!("Sender ending balance: {}", sender_ending_balance);

            assert_eq!(
                sender_starting_balance - sender_ending_balance,
                total_to_distribute
            );
        }
    });
}

#[test]
fn airdrop_delegated_single_sender_single_denom() {
    let container_name = "delegated_airdrop_test";

    run_single_node_test(container_name, |sender_account: AccountInfo| {
        async move {
            let sender_address = sender_account.address(ACCOUNT_PREFIX).unwrap();
            let delegate_account = AccountInfo::new("");
            let delegate_address = delegate_account.address(ACCOUNT_PREFIX).unwrap();

            println!("Delegate sender address {}", delegate_address);

            let recipients = generate_accounts(2);
            let payments = generate_payments(&recipients);
            let mut chain_client = init_test_chain_client().await;
            let mut txm = chain_client.get_basic_tx_metadata().await.unwrap();

            txm.gas_limit =
                MULTISEND_BASE_GAS_APPROX + (PAYMENT_GAS_APPROX * payments.len() as u64);

            // authorize MultiSend
            println!("Granting MultiSend authorization to delegate");
            let response = chain_client
                .grant_generic_authorization(
                    &sender_account,
                    delegate_account.id(ACCOUNT_PREFIX).unwrap(),
                    "/cosmos.bank.v1beta1.MsgMultiSend".to_string(),
                    Some(prost_types::Timestamp {
                        seconds: 4110314268,
                        nanos: 0,
                    }),
                    None
                )
                .await
                .unwrap();

            wait_for_tx(&chain_client.rpc_client, &response, 10).await;

            let _response = chain_client
                .query_authz_grant(
                    &sender_account.address(ACCOUNT_PREFIX).unwrap(),
                    &delegate_account.address(ACCOUNT_PREFIX).unwrap(),
                    "/cosmos.bank.v1beta1.MsgMultiSend",
                )
                .await
                .unwrap();

            // fund delegate address
            let response = chain_client
                .send(
                    &sender_account,
                    &delegate_address,
                    Coin {
                        amount: 10000000000,
                        denom: DENOM.to_string(),
                    },
                    None,
                )
                .await
                .unwrap();

            wait_for_tx(&chain_client.rpc_client, &response, 10).await;

            let total_to_distribute: u64 = payments.iter().map(|p| p.amount).sum();
            let sender_starting_balance: u64 = chain_client
                .query_all_balances(&sender_address)
                .await
                .unwrap()[0]
                .amount
                .parse()
                .unwrap();

            println!(
                "Delegated sender starting balance: {}",
                sender_starting_balance
            );
            println!("Executing delegated airdrop");
            let response = chain_client
                .execute_delegated_airdrop(
                    &sender_account.address(ACCOUNT_PREFIX).unwrap(),
                    &delegate_account,
                    payments.clone(),
                    Some(txm),
                )
                .await
                .unwrap();

            // wait 1 minute for the tx to be included in a block
            wait_for_tx(&chain_client.rpc_client, &response, 10).await;

            let sender_ending_balance: u64 = chain_client
                .query_all_balances(&sender_address)
                .await
                .unwrap()[0]
                .amount
                .parse()
                .unwrap();

            println!(
                "Delegate sender ending balance: {}",
                sender_ending_balance
            );

            assert_eq!(
                sender_starting_balance - sender_ending_balance,
                total_to_distribute
            );
        }
    });
}

fn generate_accounts(n: u64) -> Vec<AccountInfo> {
    let mut accounts = Vec::<AccountInfo>::new();

    for _ in 0..n {
        accounts.push(AccountInfo::new(""));
    }

    accounts
}

fn generate_payments(accounts: &Vec<AccountInfo>) -> Vec<Payment> {
    let mut rng = rand::thread_rng();
    accounts
        .iter()
        .map(|a| Payment {
            recipient: a.address(ACCOUNT_PREFIX).unwrap(),
            amount: rng.gen_range(1..99999),
            denom: DENOM.to_string(),
        })
        .collect()
}

async fn init_test_chain_client() -> ChainClient {
    let rpc_address = format!("http://localhost:{}", RPC_PORT);
    let rpc_client = rpc::HttpClient::new(rpc_address.as_str()).expect("Could not create RPC");

    dev::poll_for_first_block(&rpc_client).await;

    let grpc_address = format!("http://localhost:9090");
    let mut cache = Cache::create_memory_cache(None, 10).unwrap();
    let _res = cache
        .grpc_endpoint_cache
        .add_item(grpc_address.clone(), 0)
        .unwrap();

    ChainClient {
        config: ChainClientConfig {
            chain_name: "cosmrs".to_string(),
            chain_id: CHAIN_ID.to_string(),
            rpc_address: rpc_address.clone(),
            grpc_address,
            account_prefix: ACCOUNT_PREFIX.to_string(),
            gas_adjustment: 1.2,
            default_fee: ocular::tx::Coin {
                amount: 0u64,
                denom: DENOM.to_string(),
            },
        },
        keyring: Keyring::new_file_store(None).expect("Could not create keyring."),
        rpc_client: rpc_client.clone(),
        cache: Some(cache),
        connection_retry_attempts: 0,
    }
}

async fn wait_for_tx(client: &HttpClient, res: &Response, retries: u64) {
    if res.check_tx.code.is_err() {
        panic!("CheckTx error: {:?}", res);
    }

    if res.deliver_tx.code.is_err() {
        panic!("DeliverTx error: {:?}", res);
    }

    let mut result_tx: Option<Tx> = None;
    for _ in 0..retries {
        if let Ok(tx) = Tx::find_by_hash(client, res.hash).await {
            result_tx = Some(tx);
        }

        if result_tx.is_some() {
            return;
        }

        thread::sleep(Duration::from_secs(6));
    }

    panic!("timed out waiting for transaction {}", res.hash);
}
