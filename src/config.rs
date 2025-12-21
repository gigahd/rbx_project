use std::{fs, path::PathBuf};

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
	pub fn to_wally_string(self: Self) -> String {
		format!("{} = \"{}\"", self.name, self.origin)
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
	pub template_name: String,
	pub rokit_tools: Vec<String>,
	pub wally_shared_dependencies: Vec<WallyDependency>,
	pub wally_server_dependencies: Vec<WallyDependency>,
}

fn read_wally_dependencies(wally_file: PathBuf) -> (Vec<WallyDependency>, Vec<WallyDependency>) {
	let string = fs::read_to_string(wally_file).expect("Couldn't open file");
	let (_, all_dependencies) = string.split_once("[dependencies]").unwrap();
	let (shared_dependencies, server_dependencies) = all_dependencies.split_once("[server-dependencies]").unwrap();
	let mut shared_dependency_list: Vec<WallyDependency> = Vec::new();
	
	shared_dependencies.lines().for_each(|wally_string| {
		shared_dependency_list.push(WallyDependency::from_wally_string(wally_string.to_string()).unwrap()); 
	});
	let mut server_dependency_list: Vec<WallyDependency> = Vec::new();

	server_dependencies.lines().for_each(|wally_string| {
		server_dependency_list.push(WallyDependency::from_wally_string(wally_string.to_string()).unwrap()); 
	});

	(shared_dependency_list, server_dependency_list)
}

fn insert_wally_dependecy(wally_file: PathBuf, dependency: WallyDependency, is_server_dependency: bool) {
	
}

pub fn serialize_config(path: PathBuf, config: &Config) -> std::io::Result<()> {
	let toml = toml::to_string(config).expect("Failed to convert config to toml");
	create::file(path.join(CONFIG_NAME), toml.as_str())?;
	Ok(())
}

pub fn parse_config(path: PathBuf) -> std::io::Result<Config> {
	let s = fs::read_to_string(path)?;
	let toml: Config = toml::from_str(&s).expect("Failed to read config toml file");
	Ok(toml)
}