use abscissa_core::{Command, Runnable, FrameworkError, config};
use clap::Parser;

use crate::config::OcularCliConfig;

use self::bank::BankQueryCmd;

mod bank;

/// `query` subcommand
///
/// The `Parser` proc macro generates an option parser based on the struct
/// definition, and is defined in the `clap` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/clap/>
#[derive(Command, Debug, Parser, Runnable)]
pub enum QueryCmd {
    Bank(BankQueryCmd)
}

impl config::Override<OcularCliConfig> for QueryCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(&self, config: OcularCliConfig) -> Result<OcularCliConfig, FrameworkError> {
        Ok(config)
    }
}
