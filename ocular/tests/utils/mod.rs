#![allow(dead_code)]

use std::{ffi::OsStr, process, str, panic, thread, time::Duration};

use cosmrs::{dev, Tx};
use futures::Future;
use ocular::{account::AccountInfo, chain::{client::{ChainClient, cache::Cache}, config::ChainClientConfig}, keyring::Keyring};
use tendermint_rpc::{HttpClient, endpoint::broadcast::tx_commit::Response};

/// Chain ID to use for tests
pub const CHAIN_ID: &str = "cosmrs-test";

/// Gas
pub const MULTISEND_BASE_GAS_APPROX: u64 = 60000;
pub const PAYMENT_GAS_APPROX: u64 = 25000;

/// RPC port
pub const RPC_PORT: u16 = 26657;

/// Expected account numbers
// const SENDER_ACCOUNT_NUMBER: AccountNumber = 1;
// const RECIPIENT_ACCOUNT_NUMBER: AccountNumber = 9;

/// Bech32 prefix for an account
pub const ACCOUNT_PREFIX: &str = "cosmos";

/// Denom name
pub const DENOM: &str = "samoleans";

/// Example memo
// const MEMO: &str = "test memo";

// Gaia node test image built and uploaded from https://github.com/cosmos/relayer/tree/main/docker/gaiad;
// note some adaptations to file strucutre needed to build successfully (moving scripts and making directories)
// Disclaimer: Upon cosmos sdk (and other) updates, this image may need to be rebuilt and reuploaded.
pub const DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE: &str = "philipjames11/gaia-test";

pub async fn init_test_chain_client() -> ChainClient {
    let rpc_address = format!("http://localhost:{}", RPC_PORT);
    let rpc_client = HttpClient::new(rpc_address.as_str()).expect("Could not create RPC");

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

pub async fn wait_for_tx(client: &HttpClient, res: &Response, retries: u64) {
    if res.check_tx.code.is_err() {
        panic!("CheckTx error for {}", res.hash);
    }

    if res.deliver_tx.code.is_err() {
        panic!("DeliverTx error for {}", res.hash);
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

/// Execute a given `docker` command, returning what was written to stdout
/// if the command completed successfully.
///
/// Panics if the `docker` process exits with an error code.
pub fn exec_docker_command<A, S>(name: &str, args: A) -> String
where
    A: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = process::Command::new("docker")
        .arg(name)
        .args(args)
        .stdout(process::Stdio::piped())
        .output()
        .unwrap_or_else(|err| panic!("error invoking `docker {}`: {}", name, err));

    if !output.status.success() {
        panic!("`docker {}` exited with error status: {:?}", name, output);
    }

    str::from_utf8(&output.stdout)
        .expect("UTF-8 error decoding docker output")
        .trim_end()
        .to_owned()
}

/// Invoke `docker run` with the given arguments.
pub fn docker_run<A, S>(args: A) -> String
where
    A: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    exec_docker_command("run", args)
}

// Invoke `docker kill` with the given arguments.
pub fn docker_kill<A, S>(args: A) -> String
where
    A: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    exec_docker_command("kill", args)
}

// Invoke `docker kill` then `docker rm` with the given arguments.
pub fn docker_cleanup(container_name: &str) -> String
{
    let args = [container_name];
    docker_kill(args);

    let args = ["-f", container_name];
    exec_docker_command("rm", args)
}

pub fn init_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime")
}

pub(crate) fn run_single_node_test<Fut>(container_name: &str, test: fn(AccountInfo) -> Fut)
where
    Fut: Future<Output = ()>
{
    let f = || {
        init_tokio_runtime().block_on(async {
            surround(container_name, test).await
        })
    };

    match panic::catch_unwind(f){
        Ok(result) => result,
        Err(cause) => {
            docker_cleanup(container_name);
            panic::resume_unwind(cause);
        }
    };
}

async fn surround<F, Fut>(container_name: &str, test: F)
where
    F: FnOnce(AccountInfo) -> Fut,
    Fut: Future<Output = ()>
{
let sender_account = AccountInfo::new("");
    let sender_address = sender_account.address(ACCOUNT_PREFIX).unwrap();

    println!("Sender address: {}", sender_address);

    let docker_args = [
        "-d",
        "-p",
        &format!("{}:{}", RPC_PORT, RPC_PORT),
        "-p",
        &format!("{}:{}", 9090, 9090),
        "--rm",
        "--name",
        container_name,
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        &sender_address,
    ];

    docker_run(&docker_args);
    test(sender_account).await;
    docker_kill(&[container_name]);
}
