use crate::config::LensrsCliConfig;
use abscissa_core::{config, Command, FrameworkError, Runnable};
use clap::Parser;
use ocular::chain_client::ChainClient;

#[derive(Command, Debug, Parser)]
pub struct KeysAddCmd {
    name: String,
}

impl Runnable for KeysAddCmd {
    fn run(&self) {
        let client = ChainClient::default();
        //let out = client.add_key(self.name.as_str());
        //println!("{}", out);
    }
}

impl config::Override<LensrsCliConfig> for KeysAddCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(&self, config: LensrsCliConfig) -> Result<LensrsCliConfig, FrameworkError> {
        Ok(config)
    }
}
