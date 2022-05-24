use crate::prelude::*;
use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser)]
pub struct SetDefaultCmd {}

impl Runnable for SetDefaultCmd {
    /// SetDefault chain
    fn run(&self) {
        // fs open file path and append changes

        // append changes to change default_chain to chain in local directory.

        // make sure chain_name already exists. Else, tell users to add chain before setting default
    }
}
