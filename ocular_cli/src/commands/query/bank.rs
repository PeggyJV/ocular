use crate::config::OcularCliConfig;
use crate::prelude::*;
use abscissa_core::{config, Command, FrameworkError, Runnable};
use clap::Parser;
use ocular::chain_client;

#[derive(Command, Debug, Parser)]
pub struct BankQueryCmd {}

impl Runnable for BankQueryCmd {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            let client = chain_client::get_test_client().unwrap();

            client.get_bank_query_client().await.expect("failed to get bank query client");
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            std::process::exit(1);
        });
    }
}

impl config::Override<OcularCliConfig> for BankQueryCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(&self, config: OcularCliConfig) -> Result<OcularCliConfig, FrameworkError> {
        Ok(config)
    }
}
