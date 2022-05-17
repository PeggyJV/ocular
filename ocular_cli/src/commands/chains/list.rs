use crate::prelude::*;
use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser)]
pub struct ListCmd {}

impl Runnable for ListCmd {
    /// List all chains
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            match ocular::chain::registry::list_chains().await {
                Ok(info) => {
                    let info = serde_json::to_string_pretty(&info).unwrap_or_else(|err| {
                        status_err!("Can't convert string to JSON: {}", err);
                        std::process::exit(1);
                    });
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
