use std::{
    fs,
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::create;

pub const CONFIG_NAME: &str = "rbx_project.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct WallyDependency {
    pub name: String,
    pub origin: String,
}

impl WallyDependency {
    pub fn from_wally_string(wally_string: &str) -> Option<Self> {
        let (name, origin) = match wally_string.split_once('=') {
            Some(x) => x,
            None => return None,
        };
        let trimmed_name = name.trim().to_string();
        let trimmed_origin = origin.trim().replace('"', "");
        Some(WallyDependency {
            name: trimmed_name,
            origin: trimmed_origin,
        })
    }

    pub fn to_wally_string(&self) -> String {
        format!("{} = \"{}\"", self.name, self.origin)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wally {
    pub shared: Vec<WallyDependency>,
    pub server: Vec<WallyDependency>,
}

impl Wally {
    pub fn _from_wally(wally_file: &PathBuf) -> Self {
        let content = match fs::read_to_string(wally_file) {
            Ok(content) => content,
            Err(_) => {
                return Wally {
                    shared: vec![],
                    server: vec![],
                }
            }
        };

        const DEPS: &str = "[dependencies]";
        const SERVER_DEPS: &str = "[server-dependencies]";

        let (shared_dependencies, server_dependencies) =
            match (content.find(DEPS), content.find(SERVER_DEPS)) {
                // both tags present and [dependencies] appears before [server-dependencies]
                (Some(deps_pos), Some(server_pos)) if deps_pos < server_pos => {
                    let after_deps = &content[deps_pos + DEPS.len()..];
                    match after_deps.split_once(SERVER_DEPS) {
                        Some((shared, server)) => (shared.trim(), server.trim()),
                        None => (after_deps.trim(), ""),
                    }
                }

                // only [dependencies] present (or it appears after server tag)
                (Some(deps_pos), _) => {
                    let after_deps = &content[deps_pos + DEPS.len()..];
                    (after_deps.trim(), "")
                }

                // only [server-dependencies] present
                (_, Some(server_pos)) => {
                    let after_server = &content[server_pos + SERVER_DEPS.len()..];
                    ("", after_server.trim())
                }

                // neither present
                _ => ("", ""),
            };

        let mut shared_dependency_list: Vec<WallyDependency> = Vec::new();
        for wally_string in shared_dependencies.trim().lines() {
            let trimmed_wally = wally_string.trim();
            if let Some(dependency) = WallyDependency::from_wally_string(trimmed_wally) {
                shared_dependency_list.push(dependency);
            }
        }

        let mut server_dependency_list: Vec<WallyDependency> = Vec::new();
        for wally_string in server_dependencies.trim().lines() {
            let trimmed_wally = wally_string.trim();
            if let Some(dependency) = WallyDependency::from_wally_string(trimmed_wally) {
                server_dependency_list.push(dependency);
            }
        }

        Wally {
            shared: shared_dependency_list,
            server: server_dependency_list,
        }
    }

    fn convert_dependency_list(list: &[WallyDependency]) -> String {
        list.iter()
            .map(WallyDependency::to_wally_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn write_to_wally(&self, wally_file: PathBuf) -> io::Result<()> {
        let string = fs::read_to_string(&wally_file)?;
        let info = match string.split_once("[dependencies]") {
            Some((header, _)) => header.trim().to_string(),
            None => string.trim().to_string(),
        };

        let shared_string = Wally::convert_dependency_list(&self.shared);
        let server_string = Wally::convert_dependency_list(&self.server);

        let wally_string = format!(
            "{}\n\n[dependencies]\n{}\n\n[server-dependencies]\n{}",
            info, shared_string, server_string
        );

        create::file(&wally_file, &wally_string)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub template_name: String,
    pub rokit_tools: Vec<String>,
    pub wally: Option<Wally>,
}

impl Config {
    pub fn from_toml(path: &PathBuf) -> io::Result<Self> {
        let s = fs::read_to_string(path)?;
        toml::from_str(&s).map_err(|err| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to parse {}: {err}", path.display()),
            )
        })
    }

    pub fn serialize(&self, dir: &PathBuf) -> io::Result<()> {
        let toml = toml::to_string(self).map_err(|err| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to serialize config: {err}"),
            )
        })?;
        create::file(&dir.join(CONFIG_NAME), toml.as_str())?;
        Ok(())
    }
}
