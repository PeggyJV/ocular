use crate::config::OcularCliConfig;
use crate::prelude::*;
use abscissa_core::{config, Command, FrameworkError, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser)]
pub struct ShowCmd {
    name: String,
}

impl Runnable for ShowCmd {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            match ocular::chain::registry::get_chain(self.name.as_str()).await {
                Ok(info) => {
                    let info = serde_json::to_string_pretty(&info).unwrap();
                    print!("{}", info)
                }
                Err(err) => error!("{}", err),
            }
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            std::process::exit(1);
        });
    }
}

impl config::Override<OcularCliConfig> for ShowCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(&self, config: OcularCliConfig) -> Result<OcularCliConfig, FrameworkError> {
        Ok(config)
    }
}
