// Heavily borrows from https://github.com/cosmos/cosmos-rust/blob/main/cosmrs/tests/integration.rs

// Requies docker
use ocular::{
    chain::{client::cache::Cache, config::ChainClientConfig},
    cosmos_modules::*,
    keyring::Keyring,
    tx::{MultiSendIo, TxMetadata},
};

use cosmos_sdk_proto::cosmos::authz::v1beta1::{
    GenericAuthorization, MsgExec, MsgGrant, MsgRevoke,
};
use cosmrs::{
    bank::{MsgMultiSend, MsgSend},
    dev, rpc,
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    Coin,
};
use ocular::chain::client::ChainClient;
use prost::Message;

use std::{panic, str};

/// Chain ID to use for tests
const CHAIN_ID: &str = "cosmrs-test";

/// RPC port
const RPC_PORT: u16 = 26657;

/// Expected account numbers
const SENDER_ACCOUNT_NUMBER: AccountNumber = 1;
const RECIPIENT_ACCOUNT_NUMBER: AccountNumber = 9;

/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "cosmos";

/// Denom name
const DENOM: &str = "samoleans";

/// Example memo
const MEMO: &str = "test memo";

// Gaia node test image built and uploaded from https://github.com/cosmos/relayer/tree/main/docker/gaiad;
// note some adaptations to file strucutre needed to build successfully (moving scripts and making directories)
// Disclaimer: Upon cosmos sdk (and other) updates, this image may need to be rebuilt and reuploaded.
const DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE: &str = "philipjames11/gaia-test";

#[test]
fn local_single_node_chain_test() {
    let chain_id = CHAIN_ID.parse().expect("Could not parse chain id");
    let sequence_number = 0;
    let gas = 100_000;
    let timeout_height = 9001u16;

    let mut temp_ring = Keyring::new_file_store(None).expect("Could not create keyring.");
    let _sender_mnemonic = temp_ring
        .create_cosmos_key("test_only_key_override_safe", "", true)
        .expect("Could not create key");
    let sender_account = temp_ring
        .get_account("test_only_key_override_safe")
        .expect("Couldn't retrieve AccountInfo");

    let docker_args = [
        "-d",
        "-p",
        &format!("{}:{}", RPC_PORT, RPC_PORT),
        "-p",
        &format!("{}:{}", 9090, 9090),
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        &sender_account.address(ACCOUNT_PREFIX),
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
            let mut temp_ring = Keyring::new_file_store(None).expect("Could not create keyring.");
            let sender_account = temp_ring
                .get_account("test_only_key_override_safe")
                .expect("Couldn't retrieve AccountInfo");
            let _recipient_mnemonic = temp_ring
                .create_cosmos_key("test_only_key_number_2_override_safe", "", true)
                .expect("Could not create key");
            let recipient_account = temp_ring
                .get_account("test_only_key_number_2_override_safe")
                .expect("Couldn't retrieve AccountInfo");

            let _delegate_mnemonic = temp_ring
                .create_cosmos_key("test_only_delegate_key_override_safe", "", true)
                .expect("Could not create key");
            let ad_hoc_account = temp_ring
                .get_account("test_only_delegate_key_override_safe")
                .expect("Could not get public key info.");

            let amount = Coin {
                amount: 1u8.into(),
                denom: DENOM.parse().expect("Could not parse denom."),
            };
            let default_fee_amount = 0u32;
            let default_fee = Coin {
                amount: default_fee_amount.into(),
                denom: DENOM.parse().expect("Could not parse denom"),
            };
            let fee = Fee::from_amount_and_gas(default_fee.clone(), gas);

            // Expected MsgSend
            let msg_send = MsgSend {
                from_address: sender_account.id(ACCOUNT_PREFIX),
                to_address: recipient_account.id(ACCOUNT_PREFIX),
                amount: vec![amount.clone()],
            }
            .to_any()
            .expect("Could not serlialize msg.");

            let expected_tx_body = tx::Body::new(vec![msg_send], MEMO, timeout_height);
            let expected_auth_info =
                SignerInfo::single_direct(Some(sender_account.public_key()), sequence_number).auth_info(fee.clone());
            let expected_sign_doc = SignDoc::new(
                &expected_tx_body,
                &expected_auth_info,
                &chain_id,
                SENDER_ACCOUNT_NUMBER,
            )
            .expect("Could not parse sign doc.");
            let _expected_tx_raw = expected_sign_doc
                .sign(sender_account.private_key())
                .expect("Could not parse tx.");

            // Expected MsgGrant
            let msg_grant = MsgGrant {
                granter: sender_account.address(ACCOUNT_PREFIX),
                grantee: recipient_account.address(ACCOUNT_PREFIX),
                grant: Some(authz::Grant {
                    authorization: Some(prost_types::Any {
                        type_url: String::from("/cosmos.authz.v1beta1.GenericAuthorization"),
                        value: GenericAuthorization {
                            msg: String::from("/cosmos.bank.v1beta1.MsgSend"),
                        }
                        .encode_to_vec(),
                    }),
                    expiration: Some(prost_types::Timestamp {
                        seconds: 4110314268,
                        nanos: 0,
                    }),
                }),
            };

            let msg_grant = prost_types::Any {
                type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
                value: msg_grant.encode_to_vec(),
            };

            let expected_msg_grant_body = tx::Body::new(vec![msg_grant], MEMO, timeout_height);
            let expected_msg_grant_auth_info =
                SignerInfo::single_direct(Some(sender_account.public_key()), sequence_number + 1)
                    .auth_info(fee.clone());
            let expected_msg_grant_sign_doc = SignDoc::new(
                &expected_msg_grant_body,
                &expected_msg_grant_auth_info,
                &chain_id,
                SENDER_ACCOUNT_NUMBER,
            )
            .expect("Could not parse sign doc.");
            let _expected_msg_grant_raw = expected_msg_grant_sign_doc.sign(sender_account.private_key());

            // Expected MsgRevoke
            let msg_revoke = MsgRevoke {
                granter: sender_account.address(ACCOUNT_PREFIX),
                grantee: recipient_account.address(ACCOUNT_PREFIX),
                msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
            };

            let msg_revoke = prost_types::Any {
                type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
                value: msg_revoke.encode_to_vec(),
            };

            let expected_msg_revoke_body = tx::Body::new(vec![msg_revoke], MEMO, timeout_height);
            let expected_msg_revoke_auth_info =
                SignerInfo::single_direct(Some(sender_account.public_key()), sequence_number + 2)
                    .auth_info(fee.clone());
            let expected_msg_revoke_sign_doc = SignDoc::new(
                &expected_msg_revoke_body,
                &expected_msg_revoke_auth_info,
                &chain_id,
                SENDER_ACCOUNT_NUMBER,
            )
            .expect("Could not parse sign doc.");
            let _expected_msg_revoke_raw = expected_msg_revoke_sign_doc.sign(sender_account.private_key());

            // Expected MsgExec
            let mut msgs_to_execute: Vec<::prost_types::Any> = Vec::new();
            msgs_to_execute.push(
                MsgSend {
                    from_address: sender_account.id(ACCOUNT_PREFIX),
                    to_address: ad_hoc_account.id(ACCOUNT_PREFIX),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let msg_exec = MsgExec {
                grantee: recipient_account.address(ACCOUNT_PREFIX),
                msgs: msgs_to_execute,
            };

            let msg_exec = prost_types::Any {
                type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
                value: msg_exec.encode_to_vec(),
            };

            let expected_msg_exec_body = tx::Body::new(vec![msg_exec], MEMO, timeout_height);
            let expected_msg_exec_auth_info =
                SignerInfo::single_direct(Some(recipient_account.public_key()), sequence_number)
                    .auth_info(fee.clone());
            let expected_msg_exec_sign_doc = SignDoc::new(
                &expected_msg_exec_body,
                &expected_msg_exec_auth_info,
                &chain_id,
                RECIPIENT_ACCOUNT_NUMBER,
            )
            .expect("Could not parse sign doc.");
            let _expected_msg_exec_raw = expected_msg_exec_sign_doc.sign(sender_account.private_key());

            // expected MsgMultiSend
            let input_a = MultiSendIo {
                address: sender_account.address(ACCOUNT_PREFIX),
                coins: vec![ocular::tx::Coin {
                    amount: 50,
                    denom: DENOM.to_string(),
                }],
            };
            let input_b = MultiSendIo {
                address: sender_account.address(ACCOUNT_PREFIX),
                coins: vec![ocular::tx::Coin {
                    amount: 100,
                    denom: DENOM.to_string(),
                }],
            };
            let output_a = MultiSendIo {
                address: ad_hoc_account.address(ACCOUNT_PREFIX),
                coins: vec![ocular::tx::Coin {
                    amount: 75,
                    denom: DENOM.to_string(),
                }],
            };
            let output_b = MultiSendIo {
                address: recipient_account.address(ACCOUNT_PREFIX),
                coins: vec![ocular::tx::Coin {
                    amount: 75,
                    denom: DENOM.to_string(),
                }],
            };
            let inputs = vec![input_a.clone(), input_b.clone()];
            let outputs = vec![output_a.clone(), output_b.clone()];

            let msg_inputs: Vec<cosmrs::bank::MultiSendIo> = vec![
                input_a
                    .try_into()
                    .expect("couldn't convert multi send input value"),
                input_b
                    .try_into()
                    .expect("couldn't convert multi send input value"),
            ];
            let msg_outputs: Vec<cosmrs::bank::MultiSendIo> = vec![
                output_a
                    .try_into()
                    .expect("couldn't convert multi send output value"),
                output_b
                    .try_into()
                    .expect("couldn't convert multi send output value"),
            ];
            let msg_multi_send = MsgMultiSend {
                inputs: msg_inputs,
                outputs: msg_outputs,
            }
            .to_any()
            .expect("could not serialize multi send msg");
            let expected_multisend_tx_body = tx::Body::new(vec![msg_multi_send], MEMO, timeout_height);
            let expected_multisend_auth_info =
                SignerInfo::single_direct(Some(sender_account.public_key()), sequence_number + 3)
                    .auth_info(fee.clone());

            let rpc_address = format!("http://localhost:{}", RPC_PORT);
            let rpc_client =
            rpc::HttpClient::new(rpc_address.as_str()).expect("Could not create RPC");
            let grpc_address = format!("http://localhost:9090");
            let mut cache = Cache::create_memory_cache(None, 10).unwrap();
            let _res = cache.grpc_endpoint_cache.add_item(grpc_address.clone(), 0).unwrap();
            let mut chain_client = ChainClient {
                config: ChainClientConfig {
                    chain_name: "cosmrs".to_string(),
                    chain_id: chain_id.to_string(),
                    rpc_address: rpc_address.clone(),
                    grpc_address,
                    account_prefix: ACCOUNT_PREFIX.to_string(),
                    gas_adjustment: 1.2,
                    default_fee: ocular::tx::Coin { amount: default_fee_amount.into(), denom: DENOM.to_string() }
                },
                keyring: Keyring::new_file_store(None).expect("Could not create keyring."),
                rpc_client: rpc_client.clone(),
                cache: Some(cache),
                connection_retry_attempts: 0,
            };

            dev::poll_for_first_block(&rpc_client).await;

            let tx_metadata = TxMetadata {
                fee: chain_client.config.default_fee.clone(),
                fee_payer: None,
                fee_granter: None,
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            // Test MsgSend functionality
            let actual_tx_commit_response = chain_client
                .send(
                    sender_account.clone(),
                    recipient_account.id(ACCOUNT_PREFIX).as_ref(),
                    amount.clone(),
                    Some(tx_metadata.clone()),
                )
                .await;

            if let Err(err) = &actual_tx_commit_response {
                println!(
                    "msgsend failed: {:?}",
                    err
                );
            }

            let actual_tx_commit_response = actual_tx_commit_response.unwrap();
            let actual_tx = dev::poll_for_tx(&rpc_client, actual_tx_commit_response.hash).await;
            assert_eq!(&expected_tx_body, &actual_tx.body);
            assert_eq!(&expected_auth_info, &actual_tx.auth_info);

            // Test MsgGrant functionality
            let actual_msg_grant_commit_response = chain_client
                .grant_send_authorization(
                    sender_account.clone(),
                    recipient_account.id(ACCOUNT_PREFIX),
                    Some(prost_types::Timestamp {
                        seconds: 4110314268,
                        nanos: 0,
                    }),
                    tx_metadata.clone(),
                )
                .await
                .expect("Could not broadcast msg.");

            if actual_msg_grant_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msg_grant failed: {:?}",
                    actual_msg_grant_commit_response.check_tx
                );
            }

            if actual_msg_grant_commit_response.deliver_tx.code.is_err() {
                panic!(
                    "deliver_tx for msg_grant failed: {:?}",
                    actual_msg_grant_commit_response.deliver_tx
                );
            }

            let actual_msg_grant =
                dev::poll_for_tx(&rpc_client, actual_msg_grant_commit_response.hash).await;
            assert_eq!(&expected_msg_grant_body, &actual_msg_grant.body);
            assert_eq!(&expected_msg_grant_auth_info, &actual_msg_grant.auth_info);

            // Test MsgExec functionality
            let grantee_account = recipient_account;

            let mut msgs_to_send: Vec<::prost_types::Any> = Vec::new();
            msgs_to_send.push(
                MsgSend {
                    from_address: sender_account.id(ACCOUNT_PREFIX),
                    to_address: ad_hoc_account.id(ACCOUNT_PREFIX),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let actual_msg_exec_commit_response = chain_client
                .execute_authorized_tx(
                    grantee_account.clone(),
                    msgs_to_send,
                    Some(tx_metadata.clone()),
                )
                .await
                .expect("Could not broadcast msg.");

            if actual_msg_exec_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msg_exec failed: {:?}",
                    actual_msg_exec_commit_response.check_tx
                );
            }

            if actual_msg_exec_commit_response.deliver_tx.code.is_err() {
                panic!(
                    "deliver_tx for msg_exec failed: {:?}",
                    actual_msg_exec_commit_response.deliver_tx
                );
            }

            let actual_msg_exec =
                dev::poll_for_tx(&rpc_client, actual_msg_exec_commit_response.hash).await;

            assert_eq!(&expected_msg_exec_body, &actual_msg_exec.body);
            assert_eq!(&expected_msg_exec_auth_info, &actual_msg_exec.auth_info);

            // Test MsgRevoke functionality
            let actual_msg_revoke_commit_response = chain_client
                .revoke_send_authorization(
                    sender_account.clone(),
                    grantee_account.id(ACCOUNT_PREFIX),
                    tx_metadata.clone(),
                )
                .await
                .expect("Could not broadcast msg.");

            if actual_msg_revoke_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msg_revoke failed: {:?}",
                    actual_msg_revoke_commit_response.check_tx
                );
            }

            if actual_msg_revoke_commit_response.deliver_tx.code.is_err() {
                panic!(
                    "deliver_tx for msg_revoke failed: {:?}",
                    actual_msg_revoke_commit_response.deliver_tx
                );
            }

            let actual_msg_revoke =
                dev::poll_for_tx(&rpc_client, actual_msg_revoke_commit_response.hash).await;
            assert_eq!(&expected_msg_revoke_body, &actual_msg_revoke.body);
            assert_eq!(&expected_msg_revoke_auth_info, &actual_msg_revoke.auth_info);

            // Test MsgExec does not work after permissions revoked
            let mut tx_metadata_memoed = tx_metadata.clone();
            tx_metadata_memoed.memo = String::from("Exec tx #2");

            let mut msgs_to_send: Vec<::prost_types::Any> = Vec::new();
            msgs_to_send.push(
                MsgSend {
                    from_address: sender_account.id(ACCOUNT_PREFIX),
                    to_address: ad_hoc_account.id(ACCOUNT_PREFIX),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let actual_msg_exec_commit_response = chain_client
                .execute_authorized_tx(
                    grantee_account.clone(),
                    msgs_to_send,
                    Some(tx_metadata_memoed),
                )
                .await
                .expect("Could not broadcast msg.");

            if actual_msg_exec_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msg_exec failed: {:?}",
                    actual_msg_exec_commit_response. check_tx
                );
            }

            // Assert permission error since acct delegation permission was revoked
            assert_eq!(&actual_msg_exec_commit_response.deliver_tx.log.to_string()[..82], "failed to execute message; message index: 0: authorization not found: unauthorized");

            // Test MsgMultiSend functionality
            let actual_tx_commit_response = chain_client
                .multi_send(
                    sender_account.clone(),
                    inputs,
                    outputs,
                    Some(tx_metadata),
                )
                .await
                .expect("Could not broadcast msg");

            if actual_tx_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msgsend failed: {:?}",
                    actual_tx_commit_response.check_tx
                );
            }
            if actual_tx_commit_response.deliver_tx.code.is_err() {
                panic!(
                    "deliver_tx for msgsend failed: {:?}",
                    actual_tx_commit_response.deliver_tx
                );
            }
            let actual_tx = dev::poll_for_tx(&rpc_client, actual_tx_commit_response.hash).await;
            assert_eq!(&expected_multisend_tx_body, &actual_tx.body);
            assert_eq!(&expected_multisend_auth_info, &actual_tx.auth_info);
        });
    });
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime")
}
