use crate::{config, prelude::*};
use abscissa_core::{Command, Runnable};
use clap::Parser;

use std::{env, fs, path::Path, process::Command as PCommand, str};

#[derive(Command, Debug, Parser)]
pub struct EditCmd {}

impl Runnable for EditCmd {
    /// List all chains
    fn run(&self) {
        let editor = env::var("EDITOR");
        let path = config::get_config_path();
        let config_file = Path::new(path.to_str().unwrap());

        if let Ok(editor) = editor {
            PCommand::new(editor)
                .arg(&config_file)
                .status()
                .unwrap_or_else(|e| {
                    status_err!("executor exited with error: {}", e);
                    std::process::exit(1);
                });
        } else {
            PCommand::new("nano")
                .arg(&config_file)
                .status()
                .unwrap_or_else(|e| {
                    status_err!("executor exited with error: {}", e);
                    std::process::exit(1);
                });
        }

        fs::OpenOptions::new()
            .append(true)
            .open(config_file)
            .expect("Could not open file");
    }
}
