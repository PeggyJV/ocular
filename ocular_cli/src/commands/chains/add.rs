use abscissa_core::{Command, Runnable};
use clap::Parser;

#[derive(Command, Debug, Parser)]
pub struct AddCmd {}

impl Runnable for AddCmd {
    /// Add chain to local config file
    fn run(&self) {
    // Navigate to file of local config
    
    // Check if chain already exists
    // write in the file with fs:write
    }
}

