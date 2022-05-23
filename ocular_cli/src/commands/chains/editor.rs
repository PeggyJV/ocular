use crate::config;
use abscissa_core::{Command, Runnable};
use clap::Parser;

use std::{env::var, fs, path::Path, process::Command as PCommand, str};

#[derive(Command, Debug, Parser)]
pub struct EditorCmd {}

impl Runnable for EditorCmd {
    /// List all chains
    fn run(&self) {
        let editor = var("EDITOR").unwrap();
        let path = config::get_config_path();
        let config_file = Path::new(path.to_str().unwrap());

        PCommand::new(editor)
            .arg(&config_file)
            .status()
            .expect("Something went wrong");

        fs::OpenOptions::new()
            .append(true)
            .open(config_file)
            .expect("Could not open file");
    }
}
