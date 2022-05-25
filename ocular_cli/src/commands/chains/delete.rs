use crate::prelude::*;
use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser)]
pub struct DeleteCmd {}

impl Runnable for DeleteCmd {
    fn run(&self) {
        // Use hashmap to map chain name and chain info

        // Use the remove function in toml_edit crate to remove the map content from file

        // Print message that delete was successful else, print error
    }
}

