/*
 * Heavily borrows from https://github.com/cosmos/cosmos-rust/blob/main/cosmrs/tests/integration.rs
 */

// Requies docker

use ocular::{chain::config::ChainClientConfig, keyring::Keyring};

use ocular::chain::client::ChainClient;

use cosmrs::{
    bank::MsgSend,
    crypto::secp256k1,
    dev, rpc,
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    Coin,
};
use std::{panic, str};

/// Chain ID to use for tests
const CHAIN_ID: &str = "cosmrs-test";

/// RPC port
const RPC_PORT: u16 = 26657;

/// Expected account number
const ACCOUNT_NUMBER: AccountNumber = 1;

/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "cosmos";

/// Denom name
const DENOM: &str = "samoleans";

/// Example memo
const MEMO: &str = "test memo";

#[test]
fn local_single_node_chain_test() {
    let sender_private_key = secp256k1::SigningKey::random();
    let sender_public_key = sender_private_key.public_key();
    let sender_account_id = sender_public_key.account_id(ACCOUNT_PREFIX).unwrap();

    let recipient_private_key = secp256k1::SigningKey::random();
    let recipient_account_id = recipient_private_key
        .public_key()
        .account_id(ACCOUNT_PREFIX)
        .unwrap();

    let amount = Coin {
        amount: 1u8.into(),
        denom: DENOM.parse().unwrap(),
    };

    let msg_send = MsgSend {
        from_address: sender_account_id.clone(),
        to_address: recipient_account_id,
        amount: vec![amount.clone()],
    }
    .to_any()
    .unwrap();

    let chain_id = CHAIN_ID.parse().unwrap();
    let sequence_number = 0;
    let gas = 100_000;
    let fee = Fee::from_amount_and_gas(amount, gas);
    let timeout_height = 9001u16;

    let expected_tx_body = tx::Body::new(vec![msg_send], MEMO, timeout_height);
    let expected_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);
    let expected_sign_doc = SignDoc::new(&expected_tx_body, &expected_auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();
    let expected_tx_raw = expected_sign_doc.sign(&sender_private_key).unwrap();

    let docker_args = [
        "-d",
        "-p",
        &format!("{}:{}", RPC_PORT, RPC_PORT),
        dev::GAIA_DOCKER_IMAGE,
        CHAIN_ID,
        &sender_account_id.to_string(),
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
            let rpc_address = format!("http://localhost:{}", RPC_PORT);
            let rpc_client = rpc::HttpClient::new(rpc_address.as_str()).unwrap();
        
            let chain_client = ChainClient {
                config: ChainClientConfig {
                    chain_id: chain_id.to_string(),
                    rpc_address: rpc_address.clone(), 
                    grpc_address: rpc_address,
                    account_prefix: ACCOUNT_PREFIX.to_string(),
                    gas_adjustment: 1.2,
                    gas_prices: gas.to_string(),
                },
                keyring: Keyring::new_file_store(None).expect("Could not create keyring."),
                rpc_client: rpc_client.clone(),
            };

            dev::poll_for_first_block(&rpc_client).await;

            // Test MsgSend functionality 
            //let actual_tx_commit_response = chain_client.sign_and_send_msg_send(sender_account, sender_public_key, sender_private_key, recipient_account, amount, tx_metadata)



/* 
            let tx_commit_response = expected_tx_raw.broadcast_commit(&rpc_client).await.unwrap();

            if actual_tx_commit_response.check_tx.code.is_err() {
                panic!("check_tx failed: {:?}", tx_commit_response.check_tx);
            }

            if actual_tx_commit_response.deliver_tx.code.is_err() {
                panic!("deliver_tx failed: {:?}", tx_commit_response.deliver_tx);
            }

*/





            let actual_tx = dev::poll_for_tx(&rpc_client, actual_tx_commit_response.hash).await;
            assert_eq!(&expected_tx_body, &actual_tx.body);
            assert_eq!(&expected_auth_info, &actual_tx.auth_info);
        })
    });
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
