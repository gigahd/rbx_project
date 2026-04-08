use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};

use crate::{
    config::{Config, Rokit},
    create::{run_command, run_pesde_install, run_wally_type_handling},
    log_step,
};

/// Parses rokit.toml and returns a map of short tool name -> full spec.
fn parse_rokit_toml() -> Result<BTreeMap<String, String>> {
    let content = fs::read_to_string("rokit.toml")
        .context("Failed to read rokit.toml")?;

    let table: toml::Table = toml::from_str(&content)
        .context("Failed to parse rokit.toml")?;

    let tools = table
        .get("tools")
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    let mut result = BTreeMap::new();
    for (name, value) in tools {
        if let Some(spec) = value.as_str() {
            result.insert(name, spec.to_string());
        }
    }

    Ok(result)
}

fn reconcile_rokit(config: &Config) -> Result<()> {
    log_step("Reconciling rokit tools");
    let cwd = Path::new(".");

    let current_tools = if Path::new("rokit.toml").exists() {
        parse_rokit_toml()?
    } else {
        log_step("No rokit.toml found; initializing rokit");
        run_command("rokit", ["init"], cwd)?;
        BTreeMap::new()
    };

    let current_names: HashSet<&str> = current_tools.keys().map(|s| s.as_str()).collect();

    let desired: Vec<(&str, &str)> = config
        .rokit
        .tools
        .iter()
        .map(|(name, spec)| (name.as_str(), Rokit::resolve_spec(name, spec)))
        .collect();
    let desired_names: HashSet<&str> = desired.iter().map(|(name, _)| *name).collect();

    // Add or update tools
    for (short_name, desired_spec) in &desired {
        if !current_names.contains(short_name) {
            log_step(&format!("Adding rokit tool: {desired_spec}"));
            run_command("rokit", ["add", desired_spec], cwd)?;
        } else if Rokit::has_version(desired_spec) {
            // Only check for updates when a specific version is pinned.
            // Versionless specs (e.g. "lune-org/lune") mean "any version is fine".
            if let Some(current_spec) = current_tools.get(*short_name) {
                if current_spec != desired_spec {
                    log_step(&format!("Updating rokit tool: {desired_spec}"));
                    run_command("rokit", ["add", desired_spec], cwd)?;
                }
            }
        }
    }

    // Remove tools not in config
    for name in &current_names {
        if !desired_names.contains(name) {
            log_step(&format!("Removing rokit tool: {name}"));
            run_command("rokit", ["remove", name], cwd)?;
        }
    }

    Ok(())
}

fn reconcile_wally(config: &Config) -> Result<()> {
    let Some(wally) = &config.wally else {
        return Ok(());
    };
    let cwd = Path::new(".");

    log_step("Reconciling wally dependencies");

    let has_wally_tool = config.rokit.has_tool("wally");

    if !Path::new("wally.toml").exists() {
        if has_wally_tool {
            run_command("wally", ["init"], cwd)?;
        } else {
            log_step("Wally config specified but wally is not in rokit tools; skipping");
            return Ok(());
        }
    }

    wally.write_to_wally(Path::new("wally.toml"))?;

    let has_rojo = config.rokit.has_tool("rojo");

    if has_rojo && wally.has_dependencies() {
        run_wally_type_handling(cwd)?;
    }

    Ok(())
}

fn reconcile_pesde(config: &Config) -> Result<()> {
    let Some(pesde) = &config.pesde else {
        return Ok(());
    };
    let cwd = Path::new(".");

    log_step("Reconciling pesde dependencies");

    let has_pesde_tool = config.rokit.has_tool("pesde");

    if !Path::new("pesde.toml").exists() {
        if has_pesde_tool {
            run_command("pesde", ["init"], cwd)?;
        } else {
            log_step("Pesde config specified but pesde is not in rokit tools; skipping");
            return Ok(());
        }
    }

    pesde.write_to_pesde(Path::new("pesde.toml"))?;

    if pesde.has_dependencies() {
        run_pesde_install(cwd)?;
    }

    Ok(())
}

pub fn run(config: &Config) -> Result<()> {
    reconcile_rokit(config)?;
    reconcile_wally(config)?;
    reconcile_pesde(config)?;
    Ok(())
}
