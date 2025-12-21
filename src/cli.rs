use clap::{Parser, Subcommand};
use std::{env, io::Error, path::PathBuf};

use crate::{config::{self, CONFIG_NAME, Config, WallyDependency}, create::{self, run_command, run_wally_type_handling}};

const TEMPLATES: &str = "structure_templates";

#[derive(Parser, Debug)]
#[command(name = "rbx_project")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new Roblox project
    Init(ProjectArgs),

    /// Manage Wally
    Wally(ConfigArgs),

    /// Manage Rokit
    Rokit(ConfigArgs)
}

#[derive(Parser, Debug)]
pub struct ProjectArgs {
    /// Set the preset type the defaults are (default, package, plugin)
    #[arg(short, long, default_value_t = String::from("default"))]
    pub kind: String,

    /// Whether or not the project should be opened in Visual Studio Code
    #[arg(short, long, default_value_t = true)]
    pub open_in_code: bool,

    /// Output path
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    Add {
        name: String,
        //Whether the template should be changed or not
        #[arg(long, default_value_t = false)]
        global: bool,
        /// Tells wally wheter or not this is a server dependency
        #[arg(short, long, default_value_t = false)]
        is_server: bool,
    },
    Remove {
        name: String,
        //Whether the template should be changed or not
        #[arg(long, default_value_t = false)]
        global: bool,
        /// Tells wally wheter or not this is a server dependency
        #[arg(short, long, default_value_t = false)]
        is_server: bool,
    },
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

fn get_templates_dir() -> std::io::Result<PathBuf> {
    let home_dir = get_home_dir()?;
    Ok(home_dir.join(TEMPLATES))
}
fn get_template(template_name: &str) -> std::io::Result<PathBuf> {
    let templates = get_templates_dir()?;
    let template = templates.join(template_name);
    if !template.try_exists()? {
        return Err(Error::new(std::io::ErrorKind::NotFound, "Template does not exist"));
    }
    Ok(template)
}

fn get_template_config_file(config_base: &Config) -> std::io::Result<PathBuf> {
    let template_name = &config_base.template_name;
    let template = get_template(template_name)?;
    let config_file = template.join(CONFIG_NAME);
    Ok(config_file)
}

fn handle_new_command(args: ProjectArgs) -> std::io::Result<()> {
    let template = get_template(&args.kind)?;
    create::project(&args.path, &template)?;
    if args.open_in_code {
        run_command("code", ["."])?;
    }
    Ok(())
}

fn wally_add_dependency(config: &mut Config, name: &String, is_server: &bool) {
    let list = if *is_server {
        &mut config.wally.server
    } else {
        &mut config.wally.shared
    };
    //Preventing duplicates
    match list.iter().find(|dependency| dependency.name == name.split_once("=").unwrap().0.trim().to_string()) {
        Some(_) => return,
        None => {},
    }

    list.push(match WallyDependency::from_wally_string(name) {
        Some(x) => x,
        None => return,
    });
}

fn wally_remove_dependency(config: &mut Config, name: &String, is_server: &bool) {
    let list = if *is_server {
        &mut config.wally.server
    } else {
        &mut config.wally.shared
    };
    let index = match list.iter().position(|dependency| return dependency.name == *name) {
        Some(x) => x,
        None => return,
    };
    list.remove(index);
}

fn handle_wally_command(cfg: ConfigArgs) -> std::io::Result<()> {
    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;
    match cfg.action {
        ConfigAction::Add  { name, global, is_server } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                wally_add_dependency(&mut global_config, &name, &is_server);
                global_config.serialize(&template)?;
            }
            wally_add_dependency(&mut config, &name, &is_server);
        },
        ConfigAction::Remove { name, global, is_server } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                wally_remove_dependency(&mut global_config, &name, &is_server);
                global_config.serialize(&template)?;
            }
            wally_remove_dependency(&mut config, &name, &is_server);
        }
    }
    config.serialize(&PathBuf::new())?;
    config.wally.write_to_wally(PathBuf::new().join("wally.toml"))?;
    run_wally_type_handling()?;
    Ok(())
}

fn rokit_add_to_config(config: &mut Config, name: &String) {
    config.rokit_tools.push(name.clone());
}

fn rokit_remove_from_config(config: &mut Config, name: &String) -> std::io::Result<() >{
    let position = match config.rokit_tools.iter().position(|x| *x == *name) {
        Some(x) => x,
        None => return Ok(()),
    };
    config.rokit_tools.remove(position);
    Ok(())
}

fn handle_rokit_command(cfg: ConfigArgs) -> std::io::Result<()> {
    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;
    match cfg.action {
        ConfigAction::Add  { name, global, is_server: _ } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                rokit_add_to_config(&mut global_config, &name);
                global_config.serialize(&template)?;
            }
            rokit_add_to_config(&mut config, &name);
            run_command("rokit", ["add", name.as_str()])?;
        },
        ConfigAction::Remove { name, global, is_server: _ } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                rokit_remove_from_config(&mut global_config, &name)?;
                global_config.serialize(&template)?;
            }
            rokit_remove_from_config(&mut config, &name)?;
        }
    }
    config.serialize(&PathBuf::new())?;
    Ok(())
}

pub fn handle_cli(cli: Cli) -> std::io::Result<()> {
    match cli.command {
        Command::Init(args) => handle_new_command(args),
        Command::Wally(cfg) => handle_wally_command(cfg),
        Command::Rokit(cfg) => handle_rokit_command(cfg),
    }
}
