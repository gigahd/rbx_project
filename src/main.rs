mod cli;
mod create;
mod config;

use clap::Parser;

use crate::cli::handle_cli;


fn main() -> std::io::Result<()> {
    handle_cli(cli::Cli::parse())?;
    Ok(())
}
