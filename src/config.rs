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
    pub fn from_wally_string(wally_string: String) -> Option<Self> {
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
    pub fn read_from_wally(wally_file: PathBuf) -> Self {
        let string = fs::read_to_string(wally_file).expect("Couldn't open file");
        let (_, all_dependencies) = string.split_once("[dependencies]").unwrap();
        let (shared_dependencies, server_dependencies) = all_dependencies.split_once("[server-dependencies]").unwrap();
        let mut shared_dependency_list: Vec<WallyDependency> = Vec::new();
        
        shared_dependencies.trim().lines().for_each(|wally_string| {
            let trimmed_wally = wally_string.trim();
            shared_dependency_list.push(WallyDependency::from_wally_string(trimmed_wally.to_string()).unwrap()); 
        });
        let mut server_dependency_list: Vec<WallyDependency> = Vec::new();

        server_dependencies.trim().lines().for_each(|wally_string| {
            let trimmed_wally = wally_string.trim();
            server_dependency_list.push(WallyDependency::from_wally_string(trimmed_wally.to_string()).unwrap()); 
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
    pub fn from_toml(path: PathBuf) -> std::io::Result<Self> {
        let s = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&s).expect("Failed to read config toml file");
        Ok(config)
        
    }
    pub fn serialize(self: &Self, dir: PathBuf) -> std::io::Result<()> {
        let toml = toml::to_string(&self).expect("Failed to convert config to toml");
        create::file(&dir.join(CONFIG_NAME), toml.as_str())?;
        Ok(())
    }
}