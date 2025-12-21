mod cli;
mod create;
mod config;

use std::{env::{self, set_current_dir}, ffi::OsStr, fs::{self, OpenOptions}, io::{Read, Write}, path::PathBuf, process::{Command, Output}, str::FromStr};

use clap::Parser;

use crate::{cli::handle_cli, config::{Config, WallyDependency}};



// fn create_folder(folder_name: &str) -> std::io::Result<()> {
//     fs::create_dir(folder_name)
// }

// fn create_file(file_name: &str, file_content: &str) -> std::io::Result<()>{
//     let mut file = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .open(file_name)?;

//     file.write_all(file_content.as_bytes())?;
//     file.flush()?;
//     Ok(())
// }

// fn make_origin_and_move_into(main_folder_name: &str) -> std::io::Result<()> {
//     create_folder(main_folder_name)?;
//     set_current_dir(main_folder_name)?;
//     Ok(())
// }

// fn replace_file_content(file_name: &str, file_content: &str) -> std::io::Result<()> {
//     let mut file = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .truncate(true)
//         .open(file_name)?;

//     file.write_all(file_content.as_bytes())?;
//     file.flush()?;
//     Ok(())
// }

// fn read_file(file_name: &str, buf: &mut String) -> std::io::Result<usize> {
//     let mut file = OpenOptions::new()
//         .read(true)
//         .open(file_name)?;
//     file.read_to_string(buf)
// }

// fn read_project_structure(file_name: &str, project_name: &str) -> std::io::Result<String> {
//     let mut project_structure = String::new();
//     read_file(file_name, & mut project_structure)?;

//     Ok(project_structure.replace("[]", project_name))
// }

// fn initialize_script(script_path: &str, script_location: &str) -> std::io::Result<()> {
//     let mut server_initialize = String::new();
//     read_file(script_location, &mut server_initialize)?;
//     create_file(script_path, &server_initialize)?;
//     Ok(())
// }

// fn create_new_project(project_name: &str, project_structure_path: &str, initial_script_paths: [&str; 2]) -> std::io::Result<()> {
//     make_origin_and_move_into(project_name)?;
    
//     run_command("rokit", ["init"])?;
//     run_command("rokit", ["add", "rojo"])?;
//     run_command("rokit", ["add", "wally"])?;
//     run_command("rokit", ["add", "wally-package-types"])?;

//     run_command("rojo", ["init"])?;
//     run_command("wally", ["init"])?;


//     create_folder("Packages")?;
//     create_folder("ServerPackages")?;
    
//     fs::remove_file(".\\src\\client\\init.client.luau")?;
//     fs::remove_file(".\\src\\server\\init.server.luau")?;
//     fs::remove_file(".\\src\\shared\\Hello.luau")?;

//     create_folder(".\\src\\server\\Classes")?;
//     create_folder(".\\src\\server\\Services")?;
//     create_folder(".\\src\\server\\Modules")?;

//     create_folder(".\\src\\shared\\Classes")?;
//     create_folder(".\\src\\shared\\Services")?;
//     create_folder(".\\src\\shared\\SharedServices")?;
//     create_folder(".\\src\\shared\\Modules")?;

//     create_folder("Assets")?;
//     create_folder(".\\Assets\\Shared")?;
//     create_folder(".\\Assets\\Server")?;
//     create_folder(".\\Assets\\UI")?;

//     let project_structure = read_project_structure(project_structure_path, project_name)?;
//     replace_file_content("default.project.json", project_structure.as_str())?;

//     initialize_script(".\\src\\server\\init.server.luau", initial_script_paths[0])?;
//     initialize_script(".\\src\\client\\init.client.luau", initial_script_paths[1])?;
    
//     run_command("rojo", ["sourcemap", "default.project.json", "--output", "sourcemap.json"])?;
    
//     run_command("wally-package-types", ["--sourcemap", "sourcemap.json", "Packages/"])?;
    
//     create_file("selene.toml", "std = \"roblox\"")?;
//     create_file("stylua.toml", "")?;

//     replace_file_content(".gitignore", "/*.rbxlx\n/*.rbxlx.lock\n/*.rbxl.lock\nwally.lock\nsourcemap.json\nPackages/\nServerPackages/")?;
    
//     Ok(())
// }

// fn create_new_single(project_name: &str, project_structure_path: &str, initial_script_path: &str, root_name: &str) -> std::io::Result<()> {
//     make_origin_and_move_into(project_name)?;
    
//     run_command("rokit", ["init"])?;
//     run_command("rokit", ["add", "rojo"])?;
//     run_command("rokit", ["add", "wally"])?;
//     run_command("rokit", ["add", "wally-package-types"])?;

//     run_command("rojo", ["init"])?;
//     run_command("wally", ["init"])?;

//     let project_structure = read_project_structure(project_structure_path, project_name)?;
//     replace_file_content("default.project.json", project_structure.as_str())?;
    
//     fs::remove_dir_all(".\\src\\client")?;
//     fs::remove_dir_all(".\\src\\server")?;
//     fs::remove_dir_all(".\\src\\shared")?;

//     initialize_script(format!(".\\src\\{}", root_name).as_str(), initial_script_path)?;
    
//     run_command("rojo", ["sourcemap", "default.project.json", "--output", "sourcemap.json"])?;
    
//     run_command("wally-package-types", ["--sourcemap", "sourcemap.json", "Packages/"])?;
    
//     create_file("selene.toml", "std = \"roblox\"")?;
//     create_file("stylua.toml", "")?;
    
//     replace_file_content(".gitignore", "/*.rbxlx\n/*.rbxlx.lock\n/*.rbxl.lock\nwally.lock\nsourcemap.json\nPackages/\nServerPackages/")?;
    
//     Ok(())
// }

fn main() -> std::io::Result<()> {
    //handle_cli(cli::Cli::parse())?;

    config::serialize_config(PathBuf::from_str(".\\structure_templates\\default").expect("Failed to create path buffer"), &Config{
        template_name: String::from("default"),
        rokit_tools: vec!["rojo".to_string(), "wally".to_string(), "wally-package-types".to_string()],
        wally_shared_dependencies: vec![WallyDependency{ name: "t".to_string(), origin: "osyrisrblx/t@3.1.1".to_string()}, WallyDependency{ name: "Signal".to_string(), origin: "sleitnick/signal@2.0.3".to_string()}, WallyDependency{ name: "jecs".to_string(), origin: "ukendio/jecs@0.9.0".to_string()}, WallyDependency{ name: "ByteNet".to_string(), origin: "ffrostflame/bytenet@0.4.6".to_string()}],
        wally_server_dependencies: vec![WallyDependency{ name: "ProfileStore".to_string(), origin: "lm-loleris/profilestore@1.0.3".to_string()}]
    })?;

    Ok(())
}
