use ocular::{account::AccountInfo, tx::{bank::Bank, Module}};
use utils::{init_test_chain_client, wait_for_tx};

use crate::utils::{
    run_single_node_test, ACCOUNT_PREFIX, DENOM,
};

mod utils;

#[test]
fn tx_happy_path() {
    let container_name = "tx_test";

    run_single_node_test(container_name, |sender_account: AccountInfo| {
        async move {
            let recipient = AccountInfo::new("");
            let tx = Bank::Send {
                from: &sender_account.address(ACCOUNT_PREFIX).unwrap(),
                to: &recipient.address(ACCOUNT_PREFIX).unwrap(),
                amount: 100,
                denom: DENOM
            }
            .try_into_tx()
            .unwrap();
            let mut client = init_test_chain_client().await;
            let tx = tx.sign(&mut client, &sender_account).await.unwrap();
            let res = tx.broadcast(&mut client).await.unwrap();

            wait_for_tx(&client.rpc_client, &res, 10).await;
        }
    })
}
