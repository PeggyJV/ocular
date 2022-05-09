mod add;
mod delete;
mod edit;
mod editor;
mod list;
mod registry_list;
mod set_default;
mod show;
mod show_default;

use crate::config::LensrsCliConfig;
use abscissa_core::{config, Command, FrameworkError, Runnable};
use clap::Parser;

use self::list::ListCmd;
use self::show::ShowCmd;

/// `start` subcommand
///
/// The `Parser` proc macro generates an option parser based on the struct
/// definition, and is defined in the `clap` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/clap/>
#[derive(Command, Debug, Parser, Runnable)]
pub enum ChainsCmd {
    Show(ShowCmd),
    List(ListCmd),
}

impl config::Override<LensrsCliConfig> for ChainsCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(&self, config: LensrsCliConfig) -> Result<LensrsCliConfig, FrameworkError> {
        Ok(config)
    }
}
