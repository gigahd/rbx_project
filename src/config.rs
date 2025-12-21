use std::{fs, path::PathBuf, str::FromStr};

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
        let (name, origin) = match wally_string.split_once("=") {
            Some(x) => x,
            None => return None,
        };
        let trimmed_name = name.trim().to_string();
        let trimmed_origin = origin.trim().replace("\"", "");
        Some(WallyDependency { name: trimmed_name, origin: trimmed_origin })
    }
    pub fn to_wally_string(self: &Self) -> String {
        format!("{} = \"{}\"", self.name, self.origin)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wally {
    pub shared: Vec<WallyDependency>,
    pub server: Vec<WallyDependency>,
}

impl Wally {
    pub fn from_wally(wally_file: &PathBuf) -> Self {
        let content = fs::read_to_string(wally_file)
            .expect(&format!("Couldn't open file {:?}", wally_file));

        const DEPS: &str = "[dependencies]";
        const SERVER_DEPS: &str = "[server-dependencies]";

        let (shared_dependencies, server_dependencies) = match (content.find(DEPS), content.find(SERVER_DEPS)) {
            // both tags present and [dependencies] appears before [server-dependencies]
            (Some(deps_pos), Some(server_pos)) if deps_pos < server_pos => {
                let after_deps = &content[deps_pos + DEPS.len()..];
                // safe split because we know SERVER_DEPS occurs after DEPS in this branch
                let (shared, server) = after_deps.split_once(SERVER_DEPS).unwrap();
                (shared.trim(), server.trim())
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
        shared_dependencies.trim().lines().for_each(|wally_string| {
            let trimmed_wally = wally_string.trim();
            shared_dependency_list.push(WallyDependency::from_wally_string(trimmed_wally).unwrap()); 
        });
        let mut server_dependency_list: Vec<WallyDependency> = Vec::new();

        server_dependencies.trim().lines().for_each(|wally_string| {
            let trimmed_wally = wally_string.trim();
            server_dependency_list.push(WallyDependency::from_wally_string(trimmed_wally).unwrap()); 
        });

        Wally { shared: shared_dependency_list, server: server_dependency_list }
    }
    fn convert_dependency_list(list: &Vec<WallyDependency>) -> String {
        let mut string = String::new();
        list.iter().for_each(|dependency| {
            string = format!("{}\n{}", string, dependency.to_wally_string())
        });
        string
    }
    pub fn write_to_wally(self: &Self, wally_file: PathBuf) -> std::io::Result<()> {
        let string = fs::read_to_string(wally_file).expect("Couldn't open file");
        let (info, _) = string.split_once("[dependencies]").unwrap();

        let shared_string = self::Wally::convert_dependency_list(&self.shared);
        let server_string = self::Wally::convert_dependency_list(&self.server);
        
        let wally_string = format!("{}\n[dependencies]\n{}\n\n[server-dependencies]\n{}", info, shared_string, server_string);

        create::file(&PathBuf::from_str("wally.toml").expect("Failed to convert wally.toml to path buffer"), &wally_string)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub template_name: String,
    pub rokit_tools: Vec<String>,
    pub wally: Wally,
}

impl Config {
    pub fn from_toml(path: &PathBuf) -> std::io::Result<Self> {
        let s = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&s).expect("Failed to read config toml file");
        Ok(config)
        
    }
    pub fn serialize(self: &Self, dir: &PathBuf) -> std::io::Result<()> {
        let toml = toml::to_string(&self).expect("Failed to convert config to toml");
        create::file(&dir.join(CONFIG_NAME), toml.as_str())?;
        Ok(())
    }
}