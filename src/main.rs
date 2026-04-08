mod cli;
mod config;
mod create;
mod reconcile;

use clap::Parser;

use crate::cli::handle_cli;

pub fn log_step(message: &str) {
    println!("[rbx_project] {message}");
}

fn main() -> anyhow::Result<()> {
    handle_cli(cli::Cli::parse())?;
    Ok(())
}
