use crate::{config, prelude::*};
use abscissa_core::{Command, Runnable};
use clap::Parser;
use ocular::chain::{info::ChainInfo, registry};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Command, Debug, Parser)]
pub struct AddCmd {
    name: String,
}

#[derive(Deserialize, Serialize)]
struct Chains {
    default_chain: String,
    chain_data: ChainInfo,
}

impl Runnable for AddCmd {
    /// Add chain to local config file
    fn run(&self) {
        // Navigate to file of local config
        let path = config::get_config_path();
        let config_file = Path::new(path.to_str().unwrap());

        // Check if chain already exists
        let chain_content = fs::read_to_string(config_file).unwrap_or_else(|err| {
            status_err!("Can't read config file: {}", err);
            std::process::exit(1);
        });

        let chain_name = self.name.as_str();

        abscissa_tokio::run(&APP, async {
            let chain_info = registry::get_chain(chain_name).await.unwrap_or_else(|err| {
                status_err!("Can't fetch chain from chain registry: {}", err);
                std::process::exit(1);
            });

            if !chain_content.contains(chain_name) {
                // write in the file with fs:write
                let config_content = Chains {
                    default_chain: "m".to_string(),
                    chain_data: chain_info,
                };
                let config_content = toml::ser::to_string(&config_content).unwrap_or_else(|err| {
                    status_err!("{}", err);
                    std::process::exit(1);
                });

                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .open(config_file)
                    .expect("Could not open file");

                write!(file, "{}", config_content).unwrap();
            }
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            std::process::exit(1);
        });
    }
}
