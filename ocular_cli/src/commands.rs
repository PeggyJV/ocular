//! LensrsCli Subcommands

mod chains;

use self::chains::ChainsCmd;
use crate::config::LensrsCliConfig;
use abscissa_core::{config::Override, Command, Configurable, FrameworkError, Runnable};
use clap::Parser;
use std::path::PathBuf;

/// LensrsCli Subcommands
/// Subcommands need to be listed in an enum.
#[derive(Command, Debug, Parser, Runnable)]
pub enum LensrsCliCmd {
    /// command for managing chains
    #[clap(subcommand)]
    Chains(ChainsCmd),
}

/// Entry point for the application. It needs to be a struct to allow using subcommands!
#[derive(Command, Debug, Parser)]
#[clap(author, about, version)]
pub struct EntryPoint {
    #[clap(subcommand)]
    cmd: LensrsCliCmd,

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,

    /// Use the specified config file
    #[clap(short, long)]
    pub config: Option<String>,
}

impl Runnable for EntryPoint {
    fn run(&self) {
        self.cmd.run()
    }
}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<LensrsCliConfig> for EntryPoint {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        // Generate ~/.ocular/config.toml if it doesn't exist. Since abscissa does
        // not support hot-reload config yet, we have to generate this file the
        // first time ocular is run before abscissa loads in config. AFAIK this is
        // the only method that runs before config gets loaded, so I'm putting
        // it here for now.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { crate::config::init().await }).ok()
    }

    /// Apply changes to the config after it's been loaded, e.g. overriding
    /// values in a config file using command-line options.
    ///
    /// This can be safely deleted if you don't want to override config
    /// settings from command-line options.
    fn process_config(&self, config: LensrsCliConfig) -> Result<LensrsCliConfig, FrameworkError> {
        match &self.cmd {
            LensrsCliCmd::Chains(cmd) => cmd.override_config(config),
            //
            // If you don't need special overrides for some
            // subcommands, you can just use a catch all
            // _ => Ok(config),
        }
    }
}
