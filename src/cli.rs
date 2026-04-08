use clap::{Parser, Subcommand, ValueEnum};
use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    config::{self, CONFIG_NAME, Config, Pesde, PesdeDependency, Wally},
    create::{self, run_command, run_pesde_install, run_wally_type_handling},
    log_step,
    reconcile,
};

const TEMPLATES: &str = "structure_templates";
const HOME_ENV: &str = "RBX_PROJECT_HOME";

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum Realm {
    #[default]
    Shared,
    Server,
    Dev,
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

    /// List available templates
    List,

    /// Manage Wally
    Wally(ConfigArgs),

    /// Manage Rokit
    Rokit(RokitArgs),

    /// Manage pesde
    Pesde(ConfigArgs),

    /// Sync rbx_project.toml to tool config files (rokit.toml, wally.toml, pesde.toml)
    Sync,

    /// Reconcile project state with rbx_project.toml
    Reconcile,
}

#[derive(Parser, Debug)]
pub struct ProjectArgs {
    /// Template name (folder name inside structure_templates). Use `rbx_project list` to see available templates.
    #[arg(short, long, default_value = "Game")]
    pub kind: String,

    /// Do not open the project in Visual Studio Code
    #[arg(long, default_value_t = false)]
    pub no_open_in_code: bool,

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
        /// Whether the template should be changed or not
        #[arg(long, default_value_t = false)]
        global: bool,
        /// Dependency realm: shared (default), server, or dev
        #[arg(short, long, default_value = "shared")]
        realm: Realm,
    },
    Remove {
        name: String,
        /// Whether the template should be changed or not
        #[arg(long, default_value_t = false)]
        global: bool,
        /// Dependency realm: shared (default), server, or dev
        #[arg(short, long, default_value = "shared")]
        realm: Realm,
    },
    /// Reinstall packages and regenerate types/sourcemap
    Reload,
}

#[derive(Parser, Debug)]
pub struct RokitArgs {
    #[command(subcommand)]
    pub action: RokitAction,
}

#[derive(Subcommand, Debug)]
pub enum RokitAction {
    Add {
        name: String,
        /// Whether the template should be changed or not
        #[arg(long, default_value_t = false)]
        global: bool,
    },
    Remove {
        name: String,
        /// Whether the template should be changed or not
        #[arg(long, default_value_t = false)]
        global: bool,
    },
}

fn has_templates_dir(path: &Path) -> Result<bool> {
    path.join(TEMPLATES)
        .try_exists()
        .context("Failed to check for templates directory")
}

fn search_ancestors_for_home(start: &Path) -> Result<Option<PathBuf>> {
    for ancestor in start.ancestors() {
        if has_templates_dir(ancestor)? {
            return Ok(Some(ancestor.to_path_buf()));
        }
    }
    Ok(None)
}

fn get_home_dir() -> Result<PathBuf> {
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

    bail!(
        "Couldn't locate `{}`. Set {} to your rbx_project root path.",
        TEMPLATES,
        HOME_ENV
    );
}

fn get_templates_dir() -> Result<PathBuf> {
    let home_dir = get_home_dir()?;
    Ok(home_dir.join(TEMPLATES))
}

fn get_template(template_name: &str) -> Result<PathBuf> {
    let templates = get_templates_dir()?;
    let template = templates.join(template_name);
    if !template.try_exists()? {
        bail!("Template `{template_name}` does not exist");
    }
    Ok(template)
}

fn get_template_config_file(config_base: &Config) -> Result<PathBuf> {
    let template_name = &config_base.template_name;
    let template = get_template(template_name)?;
    let config_file = template.join(CONFIG_NAME);
    Ok(config_file)
}

fn handle_new_command(args: ProjectArgs) -> Result<()> {
    let kind_name = &args.kind;
    log_step(format!("Creating new project from template `{kind_name}`").as_str());
    let template = get_template(kind_name)?;
    log_step(format!("Using template path {}", template.display()).as_str());
    create::project(&args.path, &template)?;

    if !args.no_open_in_code {
        log_step("Opening project in VS Code");
        run_command("code", [args.path.as_os_str()], Path::new("."))?;
    }

    Ok(())
}

fn handle_list_command() -> Result<()> {
    let templates_dir = get_templates_dir()?;
    log_step(format!("Templates in {}", templates_dir.display()).as_str());

    let mut entries: Vec<String> = fs::read_dir(&templates_dir)
        .with_context(|| format!("Failed to read templates directory {}", templates_dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    entries.sort();

    if entries.is_empty() {
        println!("  (no templates found)");
    } else {
        for name in &entries {
            let has_config = templates_dir.join(name).join(CONFIG_NAME).exists();
            let marker = if has_config { "" } else { " (missing rbx_project.toml)" };
            println!("  - {name}{marker}");
        }
    }

    Ok(())
}

fn get_realm_map<'a>(wally: &'a mut Wally, realm: &Realm) -> &'a mut std::collections::BTreeMap<String, String> {
    match realm {
        Realm::Shared => &mut wally.shared,
        Realm::Server => &mut wally.server,
        Realm::Dev => &mut wally.dev,
    }
}

fn wally_add_dependency(config: &mut Config, entry: &str, realm: &Realm) -> Result<()> {
    let (name, origin) = entry
        .split_once('=')
        .context("Dependency must be in the form `Name = \"owner/package@version\"`")?;
    let name = name.trim().to_string();
    let origin = origin.trim().replace('"', "");

    let wally = config.wally.get_or_insert(Wally::default());
    let map = get_realm_map(wally, realm);
    map.entry(name).or_insert(origin);
    Ok(())
}

fn wally_remove_dependency(config: &mut Config, name: &str, realm: &Realm) {
    let Some(wally) = &mut config.wally else {
        return;
    };
    let map = get_realm_map(wally, realm);
    map.remove(name);
}

fn handle_wally_command(cfg: ConfigArgs) -> Result<()> {
    log_step("Updating Wally configuration");
    let cwd = Path::new(".");

    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;

    match cfg.action {
        ConfigAction::Add {
            name,
            global,
            realm,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                wally_add_dependency(&mut global_config, &name, &realm)?;
                global_config.serialize(&template)?;
            }

            wally_add_dependency(&mut config, &name, &realm)?;
        }
        ConfigAction::Remove {
            name,
            global,
            realm,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                wally_remove_dependency(&mut global_config, &name, &realm);
                global_config.serialize(&template)?;
            }

            wally_remove_dependency(&mut config, &name, &realm);
        }
        ConfigAction::Reload => {
            log_step("Reloading Wally packages");
            run_wally_type_handling(cwd)?;
            return Ok(());
        }
    }

    config.serialize(cwd)?;

    let wally = config.wally.get_or_insert(Wally::default());
    wally.write_to_wally(&cwd.join("wally.toml"))?;

    run_wally_type_handling(cwd)?;
    Ok(())
}


fn pesde_add_dependency(config: &mut Config, entry: &str, realm: &Realm) -> Result<()> {
    let (alias, spec) = entry.split_once('=').context(
        "Dependency must be in the form `Alias = scope/name@version` or `Alias = wally:owner/package@version`",
    )?;
    let alias = alias.trim().to_string();
    let spec = spec.trim().replace('"', "");

    let dep = if let Some(wally_spec) = spec.strip_prefix("wally:") {
        let (pkg, version) = wally_spec
            .rsplit_once('@')
            .context("Expected format: wally:owner/package@version")?;
        PesdeDependency::WallySource {
            wally: pkg.to_string(),
            version: version.to_string(),
        }
    } else {
        let (pkg, version) = spec
            .rsplit_once('@')
            .context("Expected format: scope/name@version")?;
        PesdeDependency::Standard {
            name: pkg.to_string(),
            version: version.to_string(),
        }
    };

    let pesde = config.pesde.get_or_insert(Pesde::default());
    let map = match realm {
        Realm::Shared => &mut pesde.dependencies,
        Realm::Server => &mut pesde.peer_dependencies,
        Realm::Dev => &mut pesde.dev_dependencies,
    };
    map.entry(alias).or_insert(dep);
    Ok(())
}

fn pesde_remove_dependency(config: &mut Config, name: &str, realm: &Realm) {
    let Some(pesde) = &mut config.pesde else {
        return;
    };
    let map = match realm {
        Realm::Shared => &mut pesde.dependencies,
        Realm::Server => &mut pesde.peer_dependencies,
        Realm::Dev => &mut pesde.dev_dependencies,
    };
    map.remove(name);
}

fn handle_pesde_command(cfg: ConfigArgs) -> Result<()> {
    log_step("Updating pesde configuration");
    let cwd = Path::new(".");

    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;

    match cfg.action {
        ConfigAction::Add {
            name,
            global,
            realm,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                pesde_add_dependency(&mut global_config, &name, &realm)?;
                global_config.serialize(&template)?;
            }

            pesde_add_dependency(&mut config, &name, &realm)?;
        }
        ConfigAction::Remove {
            name,
            global,
            realm,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                pesde_remove_dependency(&mut global_config, &name, &realm);
                global_config.serialize(&template)?;
            }

            pesde_remove_dependency(&mut config, &name, &realm);
        }
        ConfigAction::Reload => {
            log_step("Reloading pesde packages");
            run_pesde_install(cwd)?;
            return Ok(());
        }
    }

    config.serialize(cwd)?;

    let pesde = config.pesde.get_or_insert(Pesde::default());
    pesde.write_to_pesde(&cwd.join("pesde.toml"))?;

    run_pesde_install(cwd)?;
    Ok(())
}

fn handle_sync_command() -> Result<()> {
    log_step("Syncing rbx_project.toml to tool configs");
    let config = Config::from_toml(&PathBuf::from(config::CONFIG_NAME))?;

    if Path::new("rokit.toml").exists() {
        log_step("Syncing rokit.toml");
        config.rokit.write_to_rokit(Path::new("rokit.toml"))?;
    }

    if let Some(wally) = &config.wally {
        if Path::new("wally.toml").exists() {
            log_step("Syncing wally.toml");
            wally.write_to_wally(Path::new("wally.toml"))?;
        }
    }

    if let Some(pesde) = &config.pesde {
        if Path::new("pesde.toml").exists() {
            log_step("Syncing pesde.toml");
            pesde.write_to_pesde(Path::new("pesde.toml"))?;
        }
    }

    log_step("Sync complete");
    Ok(())
}

fn handle_reconcile_command() -> Result<()> {
    log_step("Reconciling project state with rbx_project.toml");
    let config = Config::from_toml(&PathBuf::from(config::CONFIG_NAME))?;
    reconcile::run(&config)?;
    log_step("Reconciliation complete");
    Ok(())
}

fn handle_rokit_command(cfg: RokitArgs) -> Result<()> {
    log_step("Updating Rokit configuration");
    let cwd = Path::new(".");

    let mut config = Config::from_toml(&PathBuf::new().join(config::CONFIG_NAME))?;
    match cfg.action {
        RokitAction::Add {
            name,
            global,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                global_config.rokit.add_tool(&name);
                global_config.serialize(&template)?;
            }

            config.rokit.add_tool(&name);
            run_command("rokit", ["add", name.as_str()], cwd)?;
        }
        RokitAction::Remove {
            name,
            global,
        } => {
            if global {
                let template = get_template(&config.template_name)?;
                let global_config_file = get_template_config_file(&config)?;
                let mut global_config = Config::from_toml(&global_config_file)?;
                global_config.rokit.remove_tool(&name);
                global_config.serialize(&template)?;
            }
            config.rokit.remove_tool(&name);
        }
    }

    config.serialize(cwd)?;
    Ok(())
}

pub fn handle_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Command::New(args) => handle_new_command(args),
        Command::List => handle_list_command(),
        Command::Wally(cfg) => handle_wally_command(cfg),
        Command::Rokit(cfg) => handle_rokit_command(cfg),
        Command::Pesde(cfg) => handle_pesde_command(cfg),
        Command::Sync => handle_sync_command(),
        Command::Reconcile => handle_reconcile_command(),
    }
}
