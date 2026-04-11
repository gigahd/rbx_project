use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    config::{self, Config, Pesde, PesdeDependency, Rokit, Wally},
    log_step,
};

/// Parses rokit.toml and returns a `Rokit` with the tools map populated.
fn read_rokit() -> Result<Option<Rokit>> {
    let path = Path::new("rokit.toml");
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).context("Failed to read rokit.toml")?;
    let table: toml::Table = toml::from_str(&content).context("Failed to parse rokit.toml")?;

    let tools = table
        .get("tools")
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    let mut map = BTreeMap::new();
    for (name, value) in tools {
        if let Some(spec) = value.as_str() {
            map.insert(name, spec.to_string());
        }
    }

    Ok(Some(Rokit { tools: map }))
}

/// Parses a wally dependency section table into a BTreeMap.
fn parse_wally_deps(table: &toml::Table, section: &str) -> BTreeMap<String, String> {
    table
        .get(section)
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

/// Parses wally.toml and returns a `Wally` with dependencies populated.
fn read_wally() -> Result<Option<Wally>> {
    let path = Path::new("wally.toml");
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).context("Failed to read wally.toml")?;
    let table: toml::Table = toml::from_str(&content).context("Failed to parse wally.toml")?;

    let shared = parse_wally_deps(&table, "dependencies");
    let server = parse_wally_deps(&table, "server-dependencies");
    let dev = parse_wally_deps(&table, "dev-dependencies");

    if shared.is_empty() && server.is_empty() && dev.is_empty() {
        // wally.toml exists but has no dependencies — still return Some so the
        // section is preserved in rbx_project.toml (user may have [package] info).
        return Ok(Some(Wally::default()));
    }

    Ok(Some(Wally { shared, server, dev }))
}

/// Parses a single pesde dependency value (inline table) into a PesdeDependency.
fn parse_pesde_dep(value: &toml::Value) -> Option<PesdeDependency> {
    let t = value.as_table()?;
    if let Some(wally) = t.get("wally").and_then(|v| v.as_str()) {
        let version = t.get("version").and_then(|v| v.as_str()).unwrap_or("");
        Some(PesdeDependency::WallySource {
            wally: wally.to_string(),
            version: version.to_string(),
        })
    } else if let Some(name) = t.get("name").and_then(|v| v.as_str()) {
        let version = t.get("version").and_then(|v| v.as_str()).unwrap_or("");
        Some(PesdeDependency::Standard {
            name: name.to_string(),
            version: version.to_string(),
        })
    } else {
        None
    }
}

/// Parses a pesde dependency section table into a BTreeMap.
fn parse_pesde_deps(table: &toml::Table, section: &str) -> BTreeMap<String, PesdeDependency> {
    table
        .get(section)
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| parse_pesde_dep(v).map(|dep| (k.clone(), dep)))
                .collect()
        })
        .unwrap_or_default()
}

/// Parses pesde.toml and returns a `Pesde` with dependencies populated.
fn read_pesde() -> Result<Option<Pesde>> {
    let path = Path::new("pesde.toml");
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).context("Failed to read pesde.toml")?;
    let table: toml::Table = toml::from_str(&content).context("Failed to parse pesde.toml")?;

    let dependencies = parse_pesde_deps(&table, "dependencies");
    let peer_dependencies = parse_pesde_deps(&table, "peer_dependencies");
    let dev_dependencies = parse_pesde_deps(&table, "dev_dependencies");

    if dependencies.is_empty() && peer_dependencies.is_empty() && dev_dependencies.is_empty() {
        return Ok(Some(Pesde::default()));
    }

    Ok(Some(Pesde {
        dependencies,
        peer_dependencies,
        dev_dependencies,
    }))
}

/// Reads rokit.toml, wally.toml and pesde.toml and updates rbx_project.toml
/// to match their current state.
pub fn run() -> Result<()> {
    let config_path = PathBuf::from(config::CONFIG_NAME);
    let mut config = Config::from_toml(&config_path)?;

    // Reconcile rokit
    if let Some(rokit) = read_rokit()? {
        log_step("Reconciling rokit tools into rbx_project.toml");
        config.rokit = rokit;
    }

    // Reconcile wally
    match read_wally()? {
        Some(wally) => {
            log_step("Reconciling wally dependencies into rbx_project.toml");
            config.wally = Some(wally);
        }
        None => {
            // No wally.toml on disk — clear the section if it existed
            if config.wally.is_some() {
                log_step("wally.toml not found; removing wally section from rbx_project.toml");
                config.wally = None;
            }
        }
    }

    // Reconcile pesde
    match read_pesde()? {
        Some(pesde) => {
            log_step("Reconciling pesde dependencies into rbx_project.toml");
            config.pesde = Some(pesde);
        }
        None => {
            if config.pesde.is_some() {
                log_step("pesde.toml not found; removing pesde section from rbx_project.toml");
                config.pesde = None;
            }
        }
    }

    config.serialize(Path::new("."))?;
    Ok(())
}
