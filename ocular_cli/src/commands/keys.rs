mod add;

use abscissa_core::{config, Command, FrameworkError, Runnable};
use clap::Parser;

use crate::config::LensrsCliConfig;

use self::add::KeysAddCmd;

#[derive(Command, Debug, Parser, Runnable)]
pub enum KeysCmd {
    Add(KeysAddCmd),
}

impl config::Override<LensrsCliConfig> for KeysCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(&self, config: LensrsCliConfig) -> Result<LensrsCliConfig, FrameworkError> {
        Ok(config)
    }
}
