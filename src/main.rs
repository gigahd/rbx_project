mod cli;
mod create;
mod config;

use std::{path::PathBuf, str::FromStr};

use clap::Parser;

use crate::{cli::handle_cli, config::{Config, Wally, WallyDependency}};


fn main() -> std::io::Result<()> {
    //handle_cli(cli::Cli::parse())?;

    // Config{
    //     template_name: String::from("default"),
    //     rokit_tools: vec!["rojo".to_string(), "wally".to_string(), "wally-package-types".to_string()],
    //     wally: Wally {
    //         shared: vec![WallyDependency{ name: "t".to_string(), origin: "osyrisrblx/t@3.1.1".to_string()}, WallyDependency{ name: "Signal".to_string(), origin: "sleitnick/signal@2.0.3".to_string()}, WallyDependency{ name: "jecs".to_string(), origin: "ukendio/jecs@0.9.0".to_string()}, WallyDependency{ name: "ByteNet".to_string(), origin: "ffrostflame/bytenet@0.4.6".to_string()}],
    //         server: vec![WallyDependency{ name: "ProfileStore".to_string(), origin: "lm-loleris/profilestore@1.0.3".to_string()}]
    //     }
    // }.serialize(PathBuf::from_str(".\\structure_templates\\default").expect("Failed to create path buffer"))?;

    let config = Wally::from_wally(&PathBuf::from_str(".").unwrap().join("wally.toml"));

    println!("{:?}", config);

    Ok(())
}
