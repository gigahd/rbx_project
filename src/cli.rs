use clap::{Parser, Subcommand};
use std::{
    env,
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
};

use crate::{
    config::{self, CONFIG_NAME, Config, Wally, WallyDependency},
    create::{self, run_command, run_wally_type_handling},
};

const TEMPLATES: &str = "structure_templates";
const HOME_ENV: &str = "RBX_PROJECT_HOME";

fn log_step(message: &str) {
    println!("[rbx_project] {message}");
}

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

    /// Manage Wally
    Wally(ConfigArgs),

    /// Manage Rokit
    Rokit(ConfigArgs),
}

#[derive(Parser, Debug)]
pub struct ProjectArgs {
    /// Set the preset type the defaults are (ServiceProject, Plugin, Package, Empty)
    #[arg(short, long, default_value_t = String::from("ServiceProject"))]
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

fn has_templates_dir(path: &Path) -> io::Result<bool> {
    path.join(TEMPLATES).try_exists()
}

fn search_ancestors_for_home(start: &Path) -> io::Result<Option<PathBuf>> {
    for ancestor in start.ancestors() {
        if has_templates_dir(ancestor)? {
            return Ok(Some(ancestor.to_path_buf()));
        }
    }
    Ok(None)
}

fn get_home_dir() -> io::Result<PathBuf> {
    if let Ok(home_override) = env::var(HOME_ENV) {
        let override_path = PathBuf::from(home_override);

        if has_templates_dir(&override_path)? {
            return Ok(override_path);
        }

        if override_path.ends_with(TEMPLATES) && override_path.try_exists()? {
            if let Some(parent) = override_path.parent() {
                return Ok(parent.to_path_buf());
            }
        }
    }

    let exe = env::current_exe()?;
    if let Some(exe_dir) = exe.parent() {
        if let Some(home_dir) = search_ancestors_for_home(exe_dir)? {
            return Ok(home_dir);
        }
    }

    if let Ok(cwd) = env::current_dir() {
        if let Some(home_dir) = search_ancestors_for_home(&cwd)? {
            return Ok(home_dir);
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if has_templates_dir(&manifest_dir)? {
        return Ok(manifest_dir);
    }

    Err(Error::new(
        ErrorKind::NotFound,
        format!(
            "Couldn't locate `{}`. Set {} to your rbx_project root path.",
            TEMPLATES, HOME_ENV
        ),
    ))
}

fn get_templates_dir() -> io::Result<PathBuf> {
    let home_dir = get_home_dir()?;
    Ok(home_dir.join(TEMPLATES))
}

fn get_template(template_name: &str) -> io::Result<PathBuf> {
    let templates = get_templates_dir()?;
    let template = templates.join(template_name);
    if !template.try_exists()? {
        return Err(Error::new(
            std::io::ErrorKind::NotFound,
            format!("Template `{template_name}` does not exist"),
        ));
    }
    Ok(template)
}

fn get_template_config_file(config_base: &Config) -> io::Result<PathBuf> {
    let template_name = &config_base.template_name;
    let template = get_template(template_name)?;
    let config_file = template.join(CONFIG_NAME);
    Ok(config_file)
}

fn handle_new_command(args: ProjectArgs) -> io::Result<()> {
    log_step(format!("Creating new project from template `{}`", args.kind).as_str());
    let template = get_template(&args.kind)?;
    log_step(format!("Using template path {}", template.display()).as_str());
    create::project(&args.path, &template)?;

    if args.open_in_code {
        log_step("Opening project in VS Code");
        run_command("code", ["."])?;
    }

    Ok(())
}

fn wally_add_dependency(config: &mut Config, name: &str, is_server: bool) -> io::Result<()> {
    let dependency = WallyDependency::from_wally_string(name).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "Dependency must be in the form `name = \"owner/package@version\"`",
        )
    })?;

    let wally = config.wally.get_or_insert(Wally {
        shared: vec![],
        server: vec![],
    });

    let list = if is_server {
        &mut wally.server
    } else {
        &mut wally.shared
    };

    // Prevent duplicates by dependency name.
    if list.iter().any(|entry| entry.name == dependency.name) {
        return Ok(());
    }

    list.push(dependency);
    Ok(())
}

fn wally_remove_dependency(config: &mut Config, name: &str, is_server: bool) {
    let Some(wally) = &mut config.wally else {
        return;
    };

    let list = if is_server {
        &mut wally.server
    } else {
        &mut wally.shared
    };

    if let Some(index) = list.iter().position(|dependency| dependency.name == name) {
        list.remove(index);
    }
}

fn handle_wally_command(cfg: ConfigArgs) -> io::Result<()> {
    log_step("Updating Wally configuration");

    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;

    match cfg.action {
        ConfigAction::Add {
            name,
            global,
            is_server,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                wally_add_dependency(&mut global_config, &name, is_server)?;
                global_config.serialize(&template)?;
            }

            wally_add_dependency(&mut config, &name, is_server)?;
        }
        ConfigAction::Remove {
            name,
            global,
            is_server,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                wally_remove_dependency(&mut global_config, &name, is_server);
                global_config.serialize(&template)?;
            }

            wally_remove_dependency(&mut config, &name, is_server);
        }
    }

    config.serialize(&PathBuf::new())?;

    let wally = config.wally.get_or_insert(Wally {
        shared: vec![],
        server: vec![],
    });
    wally.write_to_wally(PathBuf::new().join("wally.toml"))?;

    run_wally_type_handling()?;
    Ok(())
}

fn rokit_add_to_config(config: &mut Config, name: &str) {
    if !config.rokit_tools.iter().any(|tool| tool == name) {
        config.rokit_tools.push(name.to_string());
    }
}

fn rokit_remove_from_config(config: &mut Config, name: &str) {
    if let Some(position) = config.rokit_tools.iter().position(|x| *x == name) {
        config.rokit_tools.remove(position);
    }
}

fn handle_rokit_command(cfg: ConfigArgs) -> io::Result<()> {
    log_step("Updating Rokit configuration");

    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;
    match cfg.action {
        ConfigAction::Add {
            name,
            global,
            is_server: _,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                rokit_add_to_config(&mut global_config, &name);
                global_config.serialize(&template)?;
            }

            rokit_add_to_config(&mut config, &name);
            run_command("rokit", ["add", name.as_str()])?;
        }
        ConfigAction::Remove {
            name,
            global,
            is_server: _,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                rokit_remove_from_config(&mut global_config, &name);
                global_config.serialize(&template)?;
            }
            rokit_remove_from_config(&mut config, &name);
        }
    }

    config.serialize(&PathBuf::new())?;
    Ok(())
}

pub fn handle_cli(cli: Cli) -> io::Result<()> {
    match cli.command {
        Command::New(args) => handle_new_command(args),
        Command::Wally(cfg) => handle_wally_command(cfg),
        Command::Rokit(cfg) => handle_rokit_command(cfg),
    }
}
