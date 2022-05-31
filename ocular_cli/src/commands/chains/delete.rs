use crate::{config, prelude::*};
use abscissa_core::{Command, Runnable};
use clap::Parser;
use ocular::chain::info::ChainInfo;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::Write, path::Path};

#[derive(Command, Debug, Parser)]
pub struct DeleteCmd {
    name: String,
}

#[derive(Deserialize)]
struct Chains {
    chains: Vec<ChainInfo>,
}

impl Runnable for DeleteCmd {
    fn run(&self) {
        let path = config::get_config_path();
        let config_file = Path::new(path.to_str().unwrap());

        let chain_list = fs::read_to_string(config_file).unwrap_or_else(|err| {
            status_err!("Can't fetch config file: {}", err);
            std::process::exit(1);
        });

        let data: Chains = toml::from_str(&chain_list).unwrap_or_else(|err| {
            status_err!("Can't fetch list of local chains:{}", err);
            std::process::exit(1);
        });

        let mut info = HashMap::new();

        for chain_info in data.chains {
            info.insert(chain_info.chain_name.clone(), chain_info);
        }

        let chains = self.name.clone();

        if let Some(chain_details) = info.get(chains.as_str()) {
            // Remove the map content from file

            info.remove_entry(&chain_details.chain_name.clone());

            for values in info.values() {
                let config_content = toml::to_string(values).unwrap_or_else(|err| {
                    status_err!("{}", err);
                    std::process::exit(1);
                });

                fs::write(config_file, config_content);
            }
        } else {
            println!("can't find chain in local registry, run the add command to add chain")
        }

        // Use hashmap to map chain name and chain info

        // Print message that delete was successful else, print error
    }
}
