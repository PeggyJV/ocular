//! OcularCli Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.
use crate::error::Error;
use abscissa_core::tracing::debug;
use dirs;
use ocular::chain::{info::ChainInfo, registry};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// OcularCli Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OcularCliConfig {
    /// Chain related config, not read in from file
    pub default_chain: String,
    /// Locally cached chains
    pub chains: Vec<ChainInfo>,
}

impl OcularCliConfig {
    /// Builds a config path in the users home directory
    pub fn get_config_path(&self) -> PathBuf {
        let mut path = dirs::home_dir().unwrap();
        path.push(".ocular");
        path.push("config.toml");

        path
    }

    /// Initializes ocular config dir and file if they do not exist.
    pub async fn init(&self) -> Result<PathBuf, Error> {
        let path = self.get_config_path();
        let config_file = Path::new(path.to_str().unwrap());
        let config_dir = config_file.parent().unwrap();
        if !config_dir.exists() {
            println!("config directory does not exist. creating!");
            if let Err(err) = std::fs::create_dir(config_dir) {
                return Err(err.into());
            }
        }
        if !config_file.exists() {
            debug!("creating config file with default chains");
            let default_chains = self.get_default_chains().await?;
            let config_content = OcularCliConfig {
                default_chain: "cosmoshub".to_string(),
                chains: default_chains,
            };
            let config_content = toml::to_string(&config_content)?;
            fs::write(config_file, config_content)?
        }

        Ok(path)
    }

    async fn get_default_chains(&self) -> Result<Vec<ChainInfo>, Error> {
        let mut chains = Vec::<ChainInfo>::with_capacity(2);

        match registry::get_chain("cosmoshub").await {
            Ok(info) => chains.push(info),
            Err(err) => return Err(err.into()),
        };
        match registry::get_chain("osmosis").await {
            Ok(info) => chains.push(info),
            Err(err) => return Err(err.into()),
        };

        Ok(chains)
    }
}
