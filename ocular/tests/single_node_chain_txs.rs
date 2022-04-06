// Heavily borrows from https://github.com/cosmos/cosmos-rust/blob/main/cosmrs/tests/integration.rs

// Requies docker
use ocular::{
    chain::{
        client::{
            automated_tx_handler::{DelegateTransaction, DelegatedToml},
            tx::{Account, TxMetadata},
        },
        config::ChainClientConfig,
    },
    cosmos_modules::*,
    keyring::Keyring,
};

use ocular::chain::client::ChainClient;

use cosmos_sdk_proto::cosmos::authz::v1beta1::{
    GenericAuthorization, MsgExec, MsgGrant, MsgRevoke,
};
use cosmrs::{
    bank::MsgSend,
    crypto::secp256k1::SigningKey,
    dev, rpc,
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    Coin,
};
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
    let sender_mnemonic = temp_ring
        .create_cosmos_key("test_only_key_override_safe", "", true)
        .expect("Could not create key");

    let sender_seed = sender_mnemonic.to_seed("");
    let path = &"m/44'/118'/0'/0/0"
        .parse::<bip32::DerivationPath>()
        .expect("Could not parse derivation path.");

    let sender_private_key: SigningKey = SigningKey::from_bytes(
        &bip32::XPrv::derive_from_path(sender_seed, path)
            .expect("Could not create key.")
            .private_key()
            .to_bytes(),
    )
    .expect("Could not create key.");

    let sender_public_key = sender_private_key.public_key();
    let sender_account_id = sender_public_key
        .account_id(ACCOUNT_PREFIX)
        .expect("Could not create account id.");

    let recipient_mnemonic = temp_ring
        .create_cosmos_key("test_only_key_number_2_override_safe", "", true)
        .expect("Could not create key");

    let recipient_seed = recipient_mnemonic.to_seed("");
    let path = &"m/44'/118'/0'/0/0"
        .parse::<bip32::DerivationPath>()
        .expect("Could not parse derivation path.");

    let recipient_private_key: SigningKey = SigningKey::from_bytes(
        &bip32::XPrv::derive_from_path(recipient_seed, path)
            .expect("Could not create key.")
            .private_key()
            .to_bytes(),
    )
    .expect("Could not create key.");

    let recipient_public_key = recipient_private_key.public_key();
    let recipient_account_id = recipient_public_key
        .account_id(ACCOUNT_PREFIX)
        .expect("Could not create account id.");

    let amount = Coin {
        amount: 1u8.into(),
        denom: DENOM.parse().expect("Could not parse denom."),
    };

    let fee = Fee::from_amount_and_gas(amount.clone(), gas);

    // Expected MsgSend
    let msg_send = MsgSend {
        from_address: sender_account_id.clone(),
        to_address: recipient_account_id.clone(),
        amount: vec![amount.clone()],
    }
    .to_any()
    .expect("Could not serlialize msg.");

    let expected_tx_body = tx::Body::new(vec![msg_send], MEMO, timeout_height);
    let expected_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee.clone());
    let expected_sign_doc = SignDoc::new(
        &expected_tx_body,
        &expected_auth_info,
        &chain_id,
        SENDER_ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_tx_raw = expected_sign_doc
        .sign(&sender_private_key)
        .expect("Could not parse tx.");

    // Expected MsgGrant
    let msg_grant = MsgGrant {
        granter: sender_account_id.to_string(),
        grantee: recipient_account_id.to_string(),
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
        SignerInfo::single_direct(Some(sender_public_key), sequence_number + 1)
            .auth_info(fee.clone());
    let expected_msg_grant_sign_doc = SignDoc::new(
        &expected_msg_grant_body,
        &expected_msg_grant_auth_info,
        &chain_id,
        SENDER_ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_grant_raw = expected_msg_grant_sign_doc.sign(&sender_private_key);

    // Expected MsgRevoke
    let msg_revoke = MsgRevoke {
        granter: sender_account_id.to_string(),
        grantee: recipient_account_id.to_string(),
        msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend"),
    };

    let msg_revoke = prost_types::Any {
        type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
        value: msg_revoke.encode_to_vec(),
    };

    let expected_msg_revoke_body = tx::Body::new(vec![msg_revoke], MEMO, timeout_height);
    let expected_msg_revoke_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number + 2)
            .auth_info(fee.clone());
    let expected_msg_revoke_sign_doc = SignDoc::new(
        &expected_msg_revoke_body,
        &expected_msg_revoke_auth_info,
        &chain_id,
        SENDER_ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_revoke_raw = expected_msg_revoke_sign_doc.sign(&sender_private_key);

    // Expected MsgExec
    let mut msgs_to_execute: Vec<::prost_types::Any> = Vec::new();
    msgs_to_execute.push(
        MsgSend {
            from_address: sender_account_id.clone(),
            to_address: recipient_account_id.clone(),
            amount: vec![amount.clone()],
        }
        .to_any()
        .expect("Could not serlialize msg."),
    );

    let msg_exec = MsgExec {
        grantee: recipient_account_id.to_string(),
        msgs: msgs_to_execute,
    };

    let msg_exec = prost_types::Any {
        type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
        value: msg_exec.encode_to_vec(),
    };

    let expected_msg_exec_body = tx::Body::new(vec![msg_exec], MEMO, timeout_height);
    let expected_msg_exec_auth_info =
        SignerInfo::single_direct(Some(recipient_public_key), sequence_number)
            .auth_info(fee.clone());
    let expected_msg_exec_sign_doc = SignDoc::new(
        &expected_msg_exec_body,
        &expected_msg_exec_auth_info,
        &chain_id,
        RECIPIENT_ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_exec_raw = expected_msg_exec_sign_doc.sign(&sender_private_key);

    // Automated tx handler delegated workflow
    let mut file = DelegatedToml::default();
    let granter_pem_path = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        + "/.ocular/keys"
        + "/test_only_key_override_safe.pem";

    file.sender.delegate_expiration_unix_seconds = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 50000,
    )
    .expect("Could not convert to i64");
    file.sender.source_private_key_path = &granter_pem_path;

    file.sender.fee_grant_expiration_unix_seconds = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 50000,
    )
    .expect("Could not convert to i64");
    file.sender.fee_grant_amount = 500_000;
    file.sender.denom = DENOM;

    file.sender.grant_account_number = SENDER_ACCOUNT_NUMBER;
    file.sender.grant_sequence_number = sequence_number + 3;
    file.sender.grant_gas_fee = 50_000;
    file.sender.grant_gas_limit = 500_000;
    file.sender.grant_timeout_height = timeout_height.into();
    file.sender.grant_memo = MEMO;

    file.sender.exec_gas_fee = 50_000;
    file.sender.exec_gas_limit = 500_000;
    file.sender.exec_timeout_height = timeout_height.into();
    file.sender.exec_memo = MEMO;

    file.transaction.push(DelegateTransaction {
        name: "A",
        destination_account: recipient_account_id.as_ref(),
        amount: 1u8.into(),
    });

    // Save toml for later use
    let toml_path = dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        + "/.ocular/keys"
        + "/delegated_test.toml";

    let toml_string = toml::to_string(&file).expect("Could not encode toml value.");
    std::fs::write(&toml_path, toml_string).expect("Could not write to file.");

    dbg!(&sender_account_id.to_string());

    let docker_args = [
        "-d",
        "-p",
        &format!("{}:{}", RPC_PORT, RPC_PORT),
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        &sender_account_id.to_string(),
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
            let rpc_address = format!("http://localhost:{}", RPC_PORT);
            let rpc_client =
                rpc::HttpClient::new(rpc_address.as_str()).expect("Could not create RPC");

            let mut chain_client = ChainClient {
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

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: SENDER_ACCOUNT_NUMBER,
                sequence_number: sequence_number,
                gas_fee: amount.clone(),
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let sender_seed = sender_mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(sender_seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            // Test MsgSend functionality
            let actual_tx_commit_response = chain_client
                .send(
                    Account {
                        id: sender_account_id.clone(),
                        public_key: sender_public_key,
                        private_key: sender_private_key,
                    },
                    recipient_account_id.clone(),
                    amount.clone(),
                    tx_metadata,
                )
                .await
                .expect("Could not broadcast msg.");

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
            assert_eq!(&expected_tx_body, &actual_tx.body);
            assert_eq!(&expected_auth_info, &actual_tx.auth_info);

            // Test MsgGrant functionality
            let sender_seed = sender_mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(sender_seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: SENDER_ACCOUNT_NUMBER,
                sequence_number: sequence_number + 1,
                gas_fee: amount.clone(),
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let actual_msg_grant_commit_response = chain_client
                .grant_send_authorization(
                    Account {
                        id: sender_account_id.clone(),
                        public_key: sender_public_key,
                        private_key: sender_private_key,
                    },
                    recipient_account_id.clone(),
                    Some(prost_types::Timestamp {
                        seconds: 4110314268,
                        nanos: 0,
                    }),
                    tx_metadata,
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
            let grantee_seed = recipient_mnemonic.to_seed("");
            let grantee_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(grantee_seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: RECIPIENT_ACCOUNT_NUMBER,
                sequence_number: sequence_number,
                gas_fee: amount.clone(),
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let mut msgs_to_send: Vec<::prost_types::Any> = Vec::new();
            msgs_to_send.push(
                MsgSend {
                    from_address: sender_account_id.clone(),
                    to_address: recipient_account_id.clone(),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let actual_msg_exec_commit_response = chain_client
                .execute_authorized_tx(
                    Account {
                        id: recipient_account_id.clone(),
                        public_key: grantee_private_key.public_key(),
                        private_key: grantee_private_key,
                    },
                    msgs_to_send,
                    tx_metadata,
                    None,
                    None
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
            let sender_seed = sender_mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(sender_seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: SENDER_ACCOUNT_NUMBER,
                sequence_number: sequence_number + 2,
                gas_fee: amount.clone(),
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let actual_msg_revoke_commit_response = chain_client
                .revoke_send_authorization(
                    Account {
                        id: sender_account_id.clone(),
                        public_key: sender_public_key,
                        private_key: sender_private_key,
                    },
                    recipient_account_id.clone(),
                    tx_metadata,
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
            let grantee_seed = recipient_mnemonic.to_seed("");
            let grantee_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(grantee_seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: RECIPIENT_ACCOUNT_NUMBER,
                sequence_number: sequence_number + 1,
                gas_fee: amount.clone(),
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: String::from("Exec tx #2"),
            };

            let mut msgs_to_send: Vec<::prost_types::Any> = Vec::new();
            msgs_to_send.push(
                MsgSend {
                    from_address: sender_account_id.clone(),
                    to_address: recipient_account_id.clone(),
                    amount: vec![amount.clone()],
                }
                .to_any()
                .expect("Could not serlialize msg."),
            );

            let actual_msg_exec_commit_response = chain_client
                .execute_authorized_tx(
                    Account {
                        id: recipient_account_id.clone(),
                        public_key: grantee_private_key.public_key(),
                        private_key: grantee_private_key,
                    },
                    msgs_to_send,
                    tx_metadata.clone(),
                    None,
                    None
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

            // Test delegated automated tx workflow
            let actual_automated_delegated_commit_response = &chain_client
                .execute_delegated_transacton_toml(toml_path)
                .await
                .expect("Could not broadcast msg.");

            if actual_automated_delegated_commit_response
                .response
                .check_tx
                .code
                .is_err()
            {
                panic!(
                    "check_tx for automated_delegated failed: {:?}",
                    actual_automated_delegated_commit_response.response.check_tx
                );
            }

            if actual_automated_delegated_commit_response
                .response
                .deliver_tx
                .code
                .is_err()
            {
                panic!(
                    "deliver_tx for automated_delegated failed: {:?}",
                    actual_automated_delegated_commit_response.response.deliver_tx
                );
            }

            // Recreate delegate key from mnemonic to comapre expected and actual msg broadcast
            let _response = chain_client.keyring.import_cosmos_key("delegate", actual_automated_delegated_commit_response.grantee_mnemonic.phrase(), "", true).expect("Could not import key.");
            let grantee_private_key = chain_client.keyring.get_key("delegate").expect("Could not get private key");
            let grantee_pub_info = chain_client.keyring.get_public_key_and_address("delegate", ACCOUNT_PREFIX).expect("Could not get public key info.");
            let grantee_acct = Account {
                id: grantee_pub_info.account,
                public_key: grantee_pub_info.public_key,
                private_key: grantee_private_key,
            };

            let automated_delegated_msg_send = MsgSend {
                from_address: sender_account_id.clone(),
                to_address: recipient_account_id.clone(),
                amount: vec![Coin{amount: 1u8.into(), denom: DENOM.parse().expect("Could not parse")}],
            }
            .to_any()
            .expect("Could not serlialize msg.");
            let mut msgs: Vec<::prost_types::Any> = Vec::new();
            msgs.push(automated_delegated_msg_send);

            let msg = MsgExec {
                grantee: grantee_acct.id.to_string(),
                msgs: msgs,
            };

            let msg_any = prost_types::Any {
                type_url: String::from("/cosmos.authz.v1beta1.MsgExec"),
                value: msg.encode_to_vec(),
            };
            let expected_automated_delegated_tx_body = tx::Body::new(vec![msg_any], MEMO, timeout_height);
            let expected_automated_delegated_auth_info =
                SignerInfo::single_direct(Some(grantee_acct.public_key), 0)
                    .auth_info(Fee {
                        amount: vec![Coin{amount: file.sender.grant_gas_fee.into(), denom: DENOM.parse().expect("Could not parse")}],
                        gas_limit: file.sender.grant_gas_limit.into(),
                        payer: Some(grantee_acct.id.clone()),
                        granter: Some(grantee_acct.id.clone()),
                    });

            let expected_automated_delegated_sign_doc = SignDoc::new(
                &expected_automated_delegated_tx_body,
                &expected_automated_delegated_auth_info,
                &chain_id,
                10,
            )
            .expect("Could not parse sign doc.");

            let _expected_tx_raw = expected_automated_delegated_sign_doc
                .sign(&grantee_acct.private_key)
                .expect("Could not parse tx.");
            let actual_automated_delegated = dev::poll_for_tx(
                &rpc_client,
                actual_automated_delegated_commit_response.response.hash,
            )
            .await;
            assert_eq!(
                &expected_automated_delegated_tx_body,
                &actual_automated_delegated.body
            );
            assert_eq!(
                &expected_automated_delegated_auth_info,
                &actual_automated_delegated.auth_info
            );
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
