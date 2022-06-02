use crate::{config, prelude::*};
use abscissa_core::{Command, Runnable};
use clap::Parser;
use ocular::chain::info::ChainInfo;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path, str};

#[derive(Command, Debug, Parser)]
pub struct ShowCmd {
    pub name: String,
}

#[derive(Deserialize)]
struct Chains {
    chains: Vec<ChainInfo>,
}

impl Runnable for ShowCmd {
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

        let mut info = HashMap::new();

        for chain_info in data.chains {
            info.insert(chain_info.chain_name.clone(), chain_info);
        }

        let chain_name = self.name.clone();

        if let Some(chain_details) = info.get(chain_name.as_str()) {
            // customize indentation for chain_info
            let buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
            let mut serialize = serde_json::Serializer::with_formatter(buf, formatter);
            chain_details.serialize(&mut serialize).unwrap();

            println!("{}", String::from_utf8(serialize.into_inner()).unwrap());
        } else {
            println!("can't find chain in local registry, run the add command to add chain")
        }
    }
}
