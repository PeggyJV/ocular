use ocular::{
    account::AccountInfo,
    tx::{Coin, Payment},
};
use rand::Rng;

use crate::utils::{
    init_test_chain_client, run_single_node_test, wait_for_tx, ACCOUNT_PREFIX, DENOM,
    MULTISEND_BASE_GAS_APPROX, PAYMENT_GAS_APPROX,
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
    let container_name = "airdrop_test";

    run_single_node_test(container_name, |sender_account: AccountInfo| {
        async move {
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
                    None,
                    None,
                )
                .await
                .unwrap();

            wait_for_tx(&chain_client.rpc_client, &response, 10).await;

            // fund delegate address
            println!("Funding delegate address");
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
            let delegate_starting_balance: u64 = chain_client
                .query_all_balances(&delegate_address)
                .await
                .unwrap()[0]
                .amount
                .parse()
                .unwrap();

            println!(
                "Delegated sender starting balance: {}",
                delegate_starting_balance
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

            let delegate_ending_balance: u64 = chain_client
                .query_all_balances(&delegate_address)
                .await
                .unwrap()[0]
                .amount
                .parse()
                .unwrap();

            println!(
                "Delegate sender ending balance: {}",
                delegate_ending_balance
            );

            assert_eq!(
                delegate_starting_balance - delegate_ending_balance,
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
