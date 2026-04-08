use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::create;

pub const CONFIG_NAME: &str = "rbx_project.toml";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Wally {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub shared: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub server: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev: BTreeMap<String, String>,
}

impl Wally {
    fn format_dependency_section(header: &str, deps: &BTreeMap<String, String>) -> String {
        let mut s = format!("[{}]\n", header);
        for (name, origin) in deps {
            s.push_str(&format!("{} = \"{}\"\n", name, origin));
        }
        s
    }

    pub fn write_to_wally(&self, wally_file: &Path) -> Result<()> {
        let content = fs::read_to_string(wally_file)
            .with_context(|| format!("Failed to read {}", wally_file.display()))?;

        // Extract the [package] header (everything before dependency sections)
        let info = content
            .split_once("[dependencies]")
            .or_else(|| content.split_once("[server-dependencies]"))
            .or_else(|| content.split_once("[dev-dependencies]"))
            .map(|(header, _)| header.trim().to_string())
            .unwrap_or_else(|| content.trim().to_string());

        let mut wally_string = info;
        wally_string.push_str("\n\n");
        wally_string.push_str(&Self::format_dependency_section("dependencies", &self.shared));
        wally_string.push('\n');
        wally_string.push_str(&Self::format_dependency_section(
            "server-dependencies",
            &self.server,
        ));
        wally_string.push('\n');
        wally_string.push_str(&Self::format_dependency_section(
            "dev-dependencies",
            &self.dev,
        ));

        create::write_file(wally_file, wally_string.trim())?;
        Ok(())
    }

    pub fn has_dependencies(&self) -> bool {
        !self.shared.is_empty() || !self.server.is_empty() || !self.dev.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PesdeDependency {
    Standard { name: String, version: String },
    WallySource { wally: String, version: String },
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Pesde {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, PesdeDependency>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub peer_dependencies: BTreeMap<String, PesdeDependency>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<String, PesdeDependency>,
}

impl Pesde {
    fn format_dep(dep: &PesdeDependency) -> String {
        match dep {
            PesdeDependency::Standard { name, version } => {
                format!("{{ name = \"{name}\", version = \"{version}\" }}")
            }
            PesdeDependency::WallySource { wally, version } => {
                format!("{{ wally = \"{wally}\", version = \"{version}\" }}")
            }
        }
    }

    fn format_dependency_section(header: &str, deps: &BTreeMap<String, PesdeDependency>) -> String {
        let mut s = format!("[{header}]\n");
        for (alias, dep) in deps {
            s.push_str(&format!("{alias} = {}\n", Self::format_dep(dep)));
        }
        s
    }

    pub fn write_to_pesde(&self, pesde_file: &Path) -> Result<()> {
        let content = fs::read_to_string(pesde_file)
            .with_context(|| format!("Failed to read {}", pesde_file.display()))?;

        // Extract everything before dependency sections
        let info = content
            .split_once("[dependencies]")
            .or_else(|| content.split_once("[peer_dependencies]"))
            .or_else(|| content.split_once("[dev_dependencies]"))
            .map(|(header, _)| header.trim().to_string())
            .unwrap_or_else(|| content.trim().to_string());

        let mut pesde_string = info;
        pesde_string.push_str("\n\n");
        pesde_string.push_str(&Self::format_dependency_section(
            "dependencies",
            &self.dependencies,
        ));
        pesde_string.push('\n');
        pesde_string.push_str(&Self::format_dependency_section(
            "peer_dependencies",
            &self.peer_dependencies,
        ));
        pesde_string.push('\n');
        pesde_string.push_str(&Self::format_dependency_section(
            "dev_dependencies",
            &self.dev_dependencies,
        ));

        create::write_file(pesde_file, pesde_string.trim())?;
        Ok(())
    }

    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
            || !self.peer_dependencies.is_empty()
            || !self.dev_dependencies.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Rokit {
    #[serde(flatten, default)]
    pub tools: BTreeMap<String, String>,
}

impl Rokit {
    /// Extracts the short tool name from a rokit spec.
    /// `"1Axen/blink@0.15.3"` -> `"blink"`
    /// `"rojo-rbx/rojo@7.7.0"` -> `"rojo"`
    /// `"rojo"` -> `"rojo"`
    pub fn short_name(spec: &str) -> &str {
        let without_version = spec.split('@').next().unwrap_or(spec);
        without_version.rsplit('/').next().unwrap_or(without_version)
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Adds a tool from a full spec like `"1Axen/blink@0.18.7"` or `"rojo-rbx/rojo"`.
    /// The short name is derived automatically as the map key.
    pub fn add_tool(&mut self, spec: &str) {
        let short = Self::short_name(spec).to_string();
        self.tools.entry(short).or_insert_with(|| spec.to_string());
    }

    pub fn remove_tool(&mut self, name: &str) {
        self.tools.remove(name);
    }

    /// Returns true if the spec pins a specific version (contains `@`).
    pub fn has_version(spec: &str) -> bool {
        spec.contains('@')
    }

    /// Resolves the effective spec for a tool.
    /// If the value is empty, the key (short name) is the spec.
    pub fn resolve_spec<'a>(name: &'a str, spec: &'a str) -> &'a str {
        if spec.is_empty() { name } else { spec }
    }

    /// Returns an iterator over resolved specs, for passing to `rokit add`.
    pub fn specs(&self) -> impl Iterator<Item = &str> {
        self.tools
            .iter()
            .map(|(name, spec)| Self::resolve_spec(name, spec))
    }

    /// Syncs the `[tools]` section of rokit.toml with our tool map.
    /// - Tools with a pinned spec overwrite the existing entry.
    /// - Tools with an empty spec are added only if missing (preserves the installed version).
    /// - Tools in rokit.toml but not in our config are removed.
    pub fn write_to_rokit(&self, rokit_file: &Path) -> Result<()> {
        let content = fs::read_to_string(rokit_file)
            .with_context(|| format!("Failed to read {}", rokit_file.display()))?;

        let mut table: toml::Table = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", rokit_file.display()))?;

        let tools_table = table
            .entry("tools")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .context("Expected [tools] to be a table in rokit.toml")?;

        let our_names: HashSet<&str> = self.tools.keys().map(|s| s.as_str()).collect();

        // Update or insert tools
        for (name, spec) in &self.tools {
            if spec.is_empty() {
                // Empty spec: only insert if the tool isn't already present
                if !tools_table.contains_key(name) {
                    tools_table.insert(name.clone(), toml::Value::String(name.clone()));
                }
            } else {
                // Explicit spec: always overwrite
                tools_table.insert(name.clone(), toml::Value::String(spec.clone()));
            }
        }

        // Remove tools not in our config
        let keys_to_remove: Vec<String> = tools_table
            .keys()
            .filter(|k| !our_names.contains(k.as_str()))
            .cloned()
            .collect();
        for k in keys_to_remove {
            tools_table.remove(&k);
        }

        let output = toml::to_string(&table).context("Failed to serialize rokit.toml")?;
        create::write_file(rokit_file, &output)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub template_name: String,
    #[serde(default)]
    pub rokit: Rokit,
    pub wally: Option<Wally>,
    pub pesde: Option<Pesde>,
}

impl Config {
    pub fn from_toml(path: &Path) -> Result<Self> {
        let s = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config {}", path.display()))?;
        toml::from_str(&s)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn serialize(&self, dir: &Path) -> Result<()> {
        let toml = toml::to_string(self)
            .context("Failed to serialize config")?;
        create::write_file(&dir.join(CONFIG_NAME), toml.as_str())?;
        Ok(())
    }
}
