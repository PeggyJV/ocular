/*
 * Heavily borrows from https://github.com/cosmos/cosmos-rust/blob/main/cosmrs/tests/integration.rs
 */

// Requies docker
use ocular::{
    chain::{
        client::tx::{Account, TxMetadata},
        config::ChainClientConfig,
    },
    keyring::Keyring,
    cosmos_modules::*,
};

use ocular::chain::client::ChainClient;

use cosmrs::{
    bank::MsgSend,
    crypto::secp256k1::SigningKey,
    dev, rpc,
    staking::{MsgDelegate, MsgUndelegate},
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    Coin,
};
use cosmos_sdk_proto::cosmos::authz::v1beta1::{
    GenericAuthorization, MsgExec, MsgGrant, MsgRevoke,
};
use prost::Message;

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
    let chain_id = CHAIN_ID.parse().expect("Could not parse chain id");
    let sequence_number = 0;
    let gas = 100_000;
    let timeout_height = 9001u16;

    let mut temp_ring = Keyring::new_file_store(None).expect("Could not create keyring.");
    let mnemonic = temp_ring
        .create_cosmos_key("test_only_key_override_safe", "", true)
        .expect("Could not create key");

    let seed = mnemonic.to_seed("");
    let path = &"m/44'/118'/0'/0/0"
        .parse::<bip32::DerivationPath>()
        .expect("Could not parse derivation path.");

    let sender_private_key: SigningKey = SigningKey::from_bytes(
        &bip32::XPrv::derive_from_path(seed, path)
            .expect("Could not create key.")
            .private_key()
            .to_bytes(),
    )
    .expect("Could not create key.");

    let sender_public_key = sender_private_key.public_key();
    let sender_account_id = sender_public_key
        .account_id(ACCOUNT_PREFIX)
        .expect("Could not create account id.");

    let recipient_private_key = SigningKey::random();
    let recipient_account_id = recipient_private_key
        .public_key()
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
        ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_tx_raw = expected_sign_doc
        .sign(&sender_private_key)
        .expect("Could not parse tx.");

    // Expected MsgDelegate
    let msg_delegate = MsgDelegate {
        delegator_address: sender_account_id.clone(),
        validator_address: recipient_account_id.clone(),
        amount: amount.clone(),
    }
    .to_any()
    .expect("Could not serlialize msg.");

    let expected_msg_delegate_body = tx::Body::new(vec![msg_delegate], MEMO, timeout_height);
    let expected_msg_delegate_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number + 1)
            .auth_info(fee.clone());
    let expected_msg_delegate_sign_doc = SignDoc::new(
        &expected_msg_delegate_body,
        &expected_msg_delegate_auth_info,
        &chain_id,
        ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_delegate_raw = expected_msg_delegate_sign_doc
        .sign(&sender_private_key)
        .expect("Could not parse tx.");

    // Expected MsgUndelegate
    let msg_undelegate = MsgUndelegate {
        delegator_address: sender_account_id.clone(),
        validator_address: recipient_account_id.clone(),
        amount: amount.clone(),
    }
    .to_any()
    .expect("Could not serlialize msg.");

    let expected_msg_undelegate_body = tx::Body::new(vec![msg_undelegate], MEMO, timeout_height);
    let expected_msg_undelegate_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number + 2)
            .auth_info(fee.clone());
    let expected_msg_undelegate_sign_doc = SignDoc::new(
        &expected_msg_undelegate_body,
        &expected_msg_undelegate_auth_info,
        &chain_id,
        ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_undelegate_raw = expected_msg_undelegate_sign_doc
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
            expiration: Some(prost_types::Timestamp{seconds: 100, nanos: 0}),
        }),
    };

    let msg_grant = prost_types::Any {
        type_url: String::from("/cosmos.authz.v1beta1.MsgGrant"),
        value: msg_grant.encode_to_vec(),
    };

    let expected_msg_grant_body = tx::Body::new(vec![msg_grant], MEMO, timeout_height);
    let expected_msg_grant_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number + 3)
            .auth_info(fee.clone());
    let expected_msg_grant_sign_doc = SignDoc::new(
        &expected_msg_grant_body,
        &expected_msg_grant_auth_info,
        &chain_id,
        ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_grant_raw = expected_msg_grant_sign_doc
        .sign(&sender_private_key);

    // Expected MsgRevoke
    let msg_revoke = MsgRevoke {
        granter: sender_account_id.to_string(),
        grantee: recipient_account_id.to_string(),
        msg_type_url: String::from("/cosmos.bank.v1beta1.MsgSend")
    };

    let msg_revoke = prost_types::Any {
        type_url: String::from("/cosmos.authz.v1beta1.MsgRevoke"),
        value: msg_revoke.encode_to_vec(),
    };

    let expected_msg_revoke_body = tx::Body::new(vec![msg_revoke], MEMO, timeout_height);
    let expected_msg_revoke_auth_info =
        SignerInfo::single_direct(Some(sender_public_key), sequence_number + 4)
            .auth_info(fee.clone());
    let expected_msg_revoke_sign_doc = SignDoc::new(
        &expected_msg_revoke_body,
        &expected_msg_revoke_auth_info,
        &chain_id,
        ACCOUNT_NUMBER,
    )
    .expect("Could not parse sign doc.");
    let _expected_msg_revoke_raw = expected_msg_revoke_sign_doc
        .sign(&sender_private_key);

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
            let rpc_client =
                rpc::HttpClient::new(rpc_address.as_str()).expect("Could not create RPC");

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

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: ACCOUNT_NUMBER,
                sequence_number: sequence_number,
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let seed = mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(seed, path)
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

            // Test MsgDelegate functionality
            let seed = mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: ACCOUNT_NUMBER,
                sequence_number: sequence_number + 1,
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let actual_msg_delegate_commit_response = chain_client
                .delegate(
                    Account {
                        id: sender_account_id.clone(),
                        public_key: sender_public_key,
                        private_key: sender_private_key,
                    },
                    recipient_account_id.clone(),
                    amount.clone(),
                    amount.clone(),
                    tx_metadata,
                )
                .await
                .expect("Could not broadcast msg.");

            if actual_msg_delegate_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msg_delegate failed: {:?}",
                    actual_msg_delegate_commit_response.check_tx
                );
            }

            // Expect error code 1 since delegator address is not funded
            if actual_msg_delegate_commit_response.deliver_tx.code
                != cosmrs::tendermint::abci::Code::Err(1)
            {
                panic!(
                    "deliver_tx for msg_delegate failed: {:?}",
                    actual_msg_delegate_commit_response.deliver_tx
                );
            }

            let actual_msg_delegate =
                dev::poll_for_tx(&rpc_client, actual_msg_delegate_commit_response.hash).await;
            assert_eq!(&expected_msg_delegate_body, &actual_msg_delegate.body);
            assert_eq!(
                &expected_msg_delegate_auth_info,
                &actual_msg_delegate.auth_info
            );

            // Test MsgUndelegate functionality
            let seed = mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: ACCOUNT_NUMBER,
                sequence_number: sequence_number + 2,
                gas_limit: gas,
                timeout_height: timeout_height.into(),
                memo: MEMO.to_string(),
            };

            let actual_msg_undelegate_commit_response = chain_client
                .undelegate(
                    Account {
                        id: sender_account_id.clone(),
                        public_key: sender_public_key,
                        private_key: sender_private_key,
                    },
                    recipient_account_id.clone(),
                    amount.clone(),
                    amount.clone(),
                    tx_metadata,
                )
                .await
                .expect("Could not broadcast msg.");

            if actual_msg_undelegate_commit_response.check_tx.code.is_err() {
                panic!(
                    "check_tx for msg_undelegate failed: {:?}",
                    actual_msg_undelegate_commit_response.check_tx
                );
            }

            // Expect error code 1 since delegator address is not funded
            if actual_msg_undelegate_commit_response.deliver_tx.code
                != cosmrs::tendermint::abci::Code::Err(1)
            {
                panic!(
                    "deliver_tx for msg_undelegate failed: {:?}",
                    actual_msg_undelegate_commit_response.deliver_tx
                );
            }

            let actual_msg_undelegate =
                dev::poll_for_tx(&rpc_client, actual_msg_undelegate_commit_response.hash).await;
            assert_eq!(&expected_msg_undelegate_body, &actual_msg_undelegate.body);
            assert_eq!(
                &expected_msg_undelegate_auth_info,
                &actual_msg_undelegate.auth_info
            );

            // Test MsgGrant functionality
            let seed = mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: ACCOUNT_NUMBER,
                sequence_number: sequence_number + 3,
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
                    Some(prost_types::Timestamp{seconds: 100, nanos: 0}),
                    amount.clone(),
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
            assert_eq!(
                &expected_msg_grant_auth_info,
                &actual_msg_grant.auth_info
            );

            // Test MsgRevoke functionality
            let seed = mnemonic.to_seed("");
            let sender_private_key: SigningKey = SigningKey::from_bytes(
                &bip32::XPrv::derive_from_path(seed, path)
                    .expect("Could not create key.")
                    .private_key()
                    .to_bytes(),
            )
            .expect("Could not create key.");

            let tx_metadata = TxMetadata {
                chain_id: chain_id.clone(),
                account_number: ACCOUNT_NUMBER,
                sequence_number: sequence_number + 4,
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
                    amount.clone(),
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
            assert_eq!(
                &expected_msg_revoke_auth_info,
                &actual_msg_revoke.auth_info
            );
                    
        })
    });
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime")
}
