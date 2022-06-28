use crate::{config, prelude::*};
use abscissa_core::{Command, Runnable};
use clap::Parser;
use ocular::registry::chain::ChainInfo;
use serde::Deserialize;
use std::{fs, path::Path, str};

#[derive(Command, Debug, Parser)]
pub struct ListCmd {}

#[derive(Deserialize)]
struct Chains {
    chains: Vec<ChainInfo>,
}

impl Runnable for ListCmd {
    /// List all chains in local config file
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

        for chain_names in data.chains {
            println!("{}", chain_names.chain_name);
        }
    }
}
