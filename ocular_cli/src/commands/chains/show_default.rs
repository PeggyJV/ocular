use crate::{config, prelude::*};
use abscissa_core::{Command, Runnable};
use clap::Parser;
use ocular::chain::info::ChainInfo;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path, str};

#[derive(Command, Debug, Parser)]
pub struct ShowDefaultCmd {

}

#[derive(Deserialize)]
struct Chains {
    default_chain: String,
    chains: Vec<ChainInfo>,
}

impl Runnable for ShowDefaultCmd {
    /// Start the application.
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

        for chain_info in data.chains.iter() {
            if chain_info.chain_name == data.default_chain {
                let info = HashMap::from([(chain_info.chain_name.clone(), &chain_info)]);

                if let Some(chain_details) = info.get(chain_info.chain_name.as_str()) {
                    // customize indentation for chain_info

                    println!("{}", serde_json::to_string_pretty(chain_details).unwrap());
                }
            }
        }
    }
}
