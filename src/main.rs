use std::{env::{self, set_current_dir}, ffi::OsStr, fs::{self, OpenOptions}, io::{Read, Write}, process::{Command, Output}};

fn run_command<T>(command: &str, args: T) -> std::io::Result<Output>
where
    T: IntoIterator,
    T::Item: AsRef<OsStr>,
{
    Command::new("cmd")
        .args(["/C", command])
        .args(args)
        .output()
}

fn create_folder(folder_name: &str) -> std::io::Result<()> {
    fs::create_dir(folder_name)
}

fn create_file(file_name: &str, file_content: &str) -> std::io::Result<()>{
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)?;

    file.write_all(file_content.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn make_origin_and_move_into(main_folder_name: &str) -> std::io::Result<()> {
    create_folder(main_folder_name)?;
    set_current_dir(main_folder_name)?;
    Ok(())
}

fn replace_file_content(file_name: &str, file_content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)?;

    file.write_all(file_content.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn read_file(file_name: &str, buf: &mut String) -> std::io::Result<usize> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(file_name)?;
    file.read_to_string(buf)
}

fn read_project_structure(file_name: &str, project_name: &str) -> std::io::Result<String> {
    let mut project_structure = String::new();
    read_file(file_name, & mut project_structure)?;

    Ok(project_structure.replace("[]", project_name))
}

fn initialize_script(script_path: &str, script_location: &str) -> std::io::Result<()> {
    let mut server_initialize = String::new();
    read_file(script_location, &mut server_initialize)?;
    create_file(script_path, &server_initialize)?;
    Ok(())
}

fn create_new_project(project_name: &str, project_structure_path: &str, initial_script_paths: [&str; 2]) -> std::io::Result<()> {
    make_origin_and_move_into(project_name)?;
    
    run_command("rokit", ["init"])?;
    run_command("rokit", ["add", "rojo"])?;
    run_command("rokit", ["add", "wally"])?;
    run_command("rokit", ["add", "wally-package-types"])?;

    run_command("rojo", ["init"])?;
    run_command("wally", ["init"])?;


    create_folder("Packages")?;
    create_folder("ServerPackages")?;

    fs::remove_file(".\\src\\client\\init.client.luau")?;
    fs::remove_file(".\\src\\server\\init.server.luau")?;
    fs::remove_file(".\\src\\shared\\Hello.luau")?;

    create_folder(".\\src\\server\\Classes")?;
    create_folder(".\\src\\server\\Services")?;
    create_folder(".\\src\\server\\Modules")?;

    create_folder(".\\src\\shared\\Classes")?;
    create_folder(".\\src\\shared\\Services")?;
    create_folder(".\\src\\shared\\SharedServices")?;
    create_folder(".\\src\\shared\\Modules")?;

    let project_structure = read_project_structure(project_structure_path, project_name)?;
    replace_file_content("default.project.json", project_structure.as_str())?;

    initialize_script(".\\src\\server\\initialize.server.luau", initial_script_paths[0])?;
    initialize_script(".\\src\\client\\initialize.client.luau", initial_script_paths[1])?;
    
    run_command("rojo", ["sourcemap", "default.project.json", "--output", "sourcemap.json"])?;
    
    run_command("wally-package-types", ["--sourcemap", "sourcemap.json", "Packages/"])?;
    
    create_file("selene.toml", "std = \"roblox\"")?;
    create_file("stylua.toml", "")?;

    replace_file_content(".gitignore", "/*.rbxlx\n/*.rbxlx.lock\n/*.rbxl.lock\nwally.lock\nsourcemap.json\nPackages/\nServerPackages/")?;
    
    Ok(())
}

fn create_new_single(project_name: &str, project_structure_path: &str, initial_script_path: &str) -> std::io::Result<()> {
    make_origin_and_move_into(project_name)?;
    
    run_command("rokit", ["init"])?;
    run_command("rokit", ["add", "rojo"])?;
    run_command("rokit", ["add", "wally"])?;
    run_command("rokit", ["add", "wally-package-types"])?;

    run_command("rojo", ["init"])?;
    run_command("wally", ["init"])?;

    let project_structure = read_project_structure(project_structure_path, project_name)?;
    replace_file_content("default.project.json", project_structure.as_str())?;
    
    fs::remove_dir_all(".\\src\\client")?;
    fs::remove_dir_all(".\\src\\server")?;
    fs::remove_dir_all(".\\src\\shared")?;

    initialize_script(".\\src\\init.luau", initial_script_path)?;
    
    run_command("rojo", ["sourcemap", "default.project.json", "--output", "sourcemap.json"])?;
    
    run_command("wally-package-types", ["--sourcemap", "sourcemap.json", "Packages/"])?;
    
    create_file("selene.toml", "std = \"roblox\"")?;
    create_file("stylua.toml", "")?;

    replace_file_content(".gitignore", "/*.rbxlx\n/*.rbxlx.lock\n/*.rbxl.lock\nwally.lock\nsourcemap.json\nPackages/\nServerPackages/")?;
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Not a valid structure\nrbxproject [new, single] [PROJECT_NAME]");
        return Ok(());
    }

    let descriptor_arg = match args.get(1) {
        Some(x) => x,
        None => {
            println!("Type needed (new, single)");
            return Ok(());
        }
    };

    let mut home_dir = env::current_exe()?;
    while !home_dir.ends_with("rbx_project") {
        match home_dir.pop() {
            true => {},
            false => {return Ok(());}
        }
    }
    let mut project_structures = home_dir.clone();
    project_structures.push("project_structures");
    let mut initial_scripts = home_dir.clone();
    initial_scripts.push("initial_scripts");

    let descriptor_arg_str = descriptor_arg.as_str();
    let project_name = args.get(2).unwrap().as_str();
    if descriptor_arg_str == "new" {
        project_structures.push("new_project_structure.json");
        
        let mut server_init = initial_scripts.clone();
        server_init.push("InitializeServer.luau");
        
        let mut client_init = initial_scripts.clone();
        client_init.push("InitializeClient.luau");
        
        let inital_scripts = [
            server_init.to_str().unwrap(),
            client_init.to_str().unwrap(),
        ];
        create_new_project(project_name, project_structures.to_str().unwrap(), inital_scripts)?;
    } else if descriptor_arg_str == "single" {
        project_structures.push("single_project_structure.json");
        initial_scripts.push("InitializePackage.luau");
        create_new_single(project_name, project_structures.to_str().unwrap(), initial_scripts.to_str().unwrap())?;
    } else {
        println!("Type needed (new, single)");
        return Ok(());
    }
    //open the current enviorment in code
    run_command("code", ["."])?;
    Ok(())
}
