use crate::prelude::*;
use abscissa_core::{Command, Runnable};
use clap::Parser;
use serde::Serialize;

#[derive(Command, Debug, Parser)]
pub struct RegistryListCmd {}

impl Runnable for RegistryListCmd {
    /// List all chains in local config file
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            match ocular::chain::registry::list_chains().await {
                Ok(mut info) => {
                    info.drain(0..1);
                    for info in info {
                        let info = info.as_str();
                        let buf = Vec::new();
                        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                        let mut serialize = serde_json::Serializer::with_formatter(buf, formatter);
                        info.serialize(&mut serialize).unwrap();

                        println!("{}", String::from_utf8(serialize.into_inner()).unwrap());
                    }
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
