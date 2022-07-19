// Heavily borrows from https://github.com/cosmos/cosmos-rust/blob/main/cosmrs/tests/integration.rs

// Requies docker
use ocular::{
    chain::{client::cache::Cache, config::ChainClientConfig},
    cosmos_modules::*,
    keyring::Keyring,
    tx::{MultiSendIo, TxMetadata}, account::AccountInfo,
};

use cosmos_sdk_proto::cosmos::authz::v1beta1::{
    GenericAuthorization, MsgExec, MsgGrant, MsgRevoke,
};
use cosmrs::{
    bank::{MsgMultiSend, MsgSend},
    dev, rpc,
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    Coin, tendermint::chain::Id,
};
use ocular::chain::client::ChainClient;
use prost::Message;

use std::{panic, str};

mod utils;

use crate::utils::{
    generate_accounts, run_single_node_test, ACCOUNT_PREFIX, CHAIN_ID, DENOM, RPC_PORT,
};

const SENDER_ACCOUNT_NUMBER: u64 = 0;
const RECIPIENT_ACCOUNT_NUMBER: u64 = 9;
const MEMO: &str = "";

#[test]
fn local_single_node_chain_test() {
    let container_name = "local_single_node_test";

    run_single_node_test(container_name, |sender_account: AccountInfo| {
        async move {
            let mut accounts = generate_accounts(2);
            let (recipient_account, ad_hoc_account) = (accounts.pop().unwrap(), accounts.pop().unwrap());

            let amount = Coin {
                amount: 1u8.into(),
                denom: DENOM.parse().expect("Could not parse denom."),
            };
            let default_fee_amount = 0u32;
            let default_fee = Coin {
                amount: default_fee_amount.into(),
                denom: DENOM.parse().expect("Could not parse denom"),
            };
            let gas = 200000;
            let fee = Fee::from_amount_and_gas(default_fee.clone(), gas);
            let sequence_number = 0;
            let timeout_height = 9001u16;
            let chain_id = Id::try_from(CHAIN_ID.to_string()).unwrap();

            // Expected MsgSend
            let msg_send = MsgSend {
                from_address: sender_account.id(ACCOUNT_PREFIX).unwrap(),
                to_address: recipient_account.id(ACCOUNT_PREFIX).unwrap(),
                amount: vec![amount.clone()],
            }
            .to_any()
            .expect("Could not serlialize msg.");

            let expected_tx_body = tx::Body::new(vec![msg_send], "", timeout_height);
            let expected_auth_info =
                SignerInfo::single_direct(Some(sender_account.public_key()), sequence_number).auth_info(fee.clone());
            let expected_sign_doc = SignDoc::new(
                &expected_tx_body,
                &expected_auth_info,
                &chain_id,
                1,
            )
            .expect("Could not parse sign doc.");
            let _expected_tx_raw = expected_sign_doc
                .sign(sender_account.private_key())
                .expect("Could not parse tx.");

            // Expected MsgGrant
            let msg_grant = MsgGrant {
                granter: sender_account.address(ACCOUNT_PREFIX).unwrap(),
                grantee: recipient_account.address(ACCOUNT_PREFIX).unwrap(),
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

            let expected_msg_grant_body = tx::Body::new(vec![msg_grant], "", timeout_height);
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
                granter: sender_account.address(ACCOUNT_PREFIX).unwrap(),
                grantee: recipient_account.address(ACCOUNT_PREFIX).unwrap(),
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
                    from_address: sender_account.id(ACCOUNT_PREFIX).unwrap(),
                    to_address: ad_hoc_account.id(ACCOUNT_PREFIX).unwrap(),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let msg_exec = MsgExec {
                grantee: recipient_account.address(ACCOUNT_PREFIX).unwrap(),
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
                address: sender_account.address(ACCOUNT_PREFIX).unwrap(),
                coins: vec![ocular::tx::Coin {
                    amount: 50,
                    denom: DENOM.to_string(),
                }],
            };
            let input_b = MultiSendIo {
                address: sender_account.address(ACCOUNT_PREFIX).unwrap(),
                coins: vec![ocular::tx::Coin {
                    amount: 100,
                    denom: DENOM.to_string(),
                }],
            };
            let output_a = MultiSendIo {
                address: ad_hoc_account.address(ACCOUNT_PREFIX).unwrap(),
                coins: vec![ocular::tx::Coin {
                    amount: 75,
                    denom: DENOM.to_string(),
                }],
            };
            let output_b = MultiSendIo {
                address: recipient_account.address(ACCOUNT_PREFIX).unwrap(),
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
                    &sender_account,
                    recipient_account.id(ACCOUNT_PREFIX).unwrap().as_ref(),
                    ocular::tx::Coin { amount: 1, denom: DENOM.to_string() },
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
                .grant_generic_authorization(
                    &sender_account,
                    recipient_account.id(ACCOUNT_PREFIX).unwrap(),
                    "/cosmos.bank.v1beta1.MsgSend",
                    Some(prost_types::Timestamp {
                        seconds: 4110314268,
                        nanos: 0,
                    }),
                    Some(tx_metadata.clone()),
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
                    from_address: sender_account.id(ACCOUNT_PREFIX).unwrap(),
                    to_address: ad_hoc_account.id(ACCOUNT_PREFIX).unwrap(),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let actual_msg_exec_commit_response = chain_client
                .execute_authorized_tx(
                    &grantee_account,
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
                    &sender_account,
                    grantee_account.id(ACCOUNT_PREFIX).unwrap(),
                    Some(tx_metadata.clone()),
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
                    from_address: sender_account.id(ACCOUNT_PREFIX).unwrap(),
                    to_address: ad_hoc_account.id(ACCOUNT_PREFIX).unwrap(),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let actual_msg_exec_commit_response = chain_client
                .execute_authorized_tx(
                    &grantee_account,
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
                    &sender_account,
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
        }
    });
}
