use clap::{Parser, Subcommand, ValueEnum};
use std::{env, io::Error, path::PathBuf};

use crate::create;

const TEMPLATES: &str = "./sturctue_templates";

#[derive(Parser, Debug)]
#[command(name = "rbx_project")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new Roblox project
    New(ProjectArgs),

    /// Manage global configuration
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
pub struct ProjectArgs {
    #[arg(value_enum)]
    pub kind: ProjectKind,

    /// Output path
    pub path: PathBuf,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ProjectKind {
    New,
    Package,
    Plugin,
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    Add {
        manager: DepManagerCli,
        name: String,
    },
    Remove {
        manager: DepManagerCli,
        name: String,
    },
    List,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DepManagerCli {
    Rokit,
    Wally,
}

fn get_home_dir() -> std::io::Result<PathBuf> {
    let mut home_dir = env::current_exe()?;
    while !home_dir.ends_with("rbx_project") {
        match home_dir.pop() {
            true => {},
            false => {return Err(Error::new(std::io::ErrorKind::NotADirectory, "Couldn't find home directory"));}
        }
    }
    Ok(home_dir)
}

fn handle_new_command(args: ProjectArgs) -> std::io::Result<()> {
    let home_dir = get_home_dir()?;
    let template_dir = home_dir.join(TEMPLATES);
    match args.kind {
        ProjectKind::New => create::project(&args.path, &template_dir.join("default")),
        ProjectKind::Plugin => create::project(&args.path, &template_dir.join("plugin")),
        ProjectKind::Package => create::project(&args.path, &template_dir.join("package"))
    }?;
    Ok(())
}

fn handle_config_command(cfg: ConfigArgs) -> std::io::Result<()> {
    println!("Config needs to be implemented");
    Ok(())
}

pub fn handle_cli(cli: Cli) -> std::io::Result<()> {
    match cli.command {
        Command::New(args) => handle_new_command(args),
        Command::Config(cfg) => handle_config_command(cfg),
    }
}
