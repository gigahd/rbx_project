use std::{env::set_current_dir, ffi::OsStr, fs::{self, OpenOptions}, io::{self, Error, Write}, path::{Path, PathBuf}, process::{Command, Output}, str::FromStr};

use crate::config::{self, Wally};

pub fn run_command<T>(command: &str, args: T) -> std::io::Result<Output>
where
    T: IntoIterator,
    T::Item: AsRef<OsStr>,
{
    Command::new("cmd")
        .args(["/C", command])
        .args(args)
        .output()
}

fn is_text_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("toml" | "md" | "txt" | "yaml" | "yml" | "json" | "sh")
    )
}

/// Recursively copies a directory.
/// Assumes no symlinks and overwrites existing files
fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    project_name: &str,
) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path, project_name)?;
        } else if ty.is_file() {
            if is_text_file(&src_path) {
                let contents = fs::read_to_string(&src_path)?;
                let rendered = contents.replace("{{project_name}}", project_name);
                fs::write(&dst_path, rendered)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}

pub fn file(file_name: &PathBuf, file_content: &str) -> std::io::Result<()> {
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

fn make_origin_and_move_into(main_folder_name: &PathBuf) -> std::io::Result<()> {
    folder(main_folder_name)?;
    set_current_dir(main_folder_name)?;
    Ok(())
}

fn folder(folder_name: &PathBuf) -> std::io::Result<()> {
    fs::create_dir(folder_name)
}

fn initialize_empty_rojo() -> std::io::Result<()> {
    run_command("rojo", ["init", "--kind", "model"])?;
    //Removes the only created file to just have an empty source
    let path_buf = PathBuf::new().join("src").join("init.luau");
    fs::remove_file(&path_buf)?;
    Ok(())
}

fn initialize_wally(wally_dependencies: &Wally) -> std::io::Result<()> {
    run_command("wally", ["init"])?;
    wally_dependencies.write_to_wally(PathBuf::from_str("./wally.toml").expect("Failed to find wally.toml"))?;
    Ok(())
}

pub fn run_wally_type_handling() -> std::io::Result<()> {
    run_command("wally", ["install"])?;
    run_command("rojo", ["sourcemap", "default.project.json", "--output", "sourcemap.json"])?;
    run_command("wally-package-types", ["--sourcemap", "sourcemap.json", "Packages/"])?;
    Ok(())
}

pub fn project(output: &PathBuf, template: &PathBuf) -> std::io::Result<()> {
    
    println!("{:?}", template);
    let template_config = config::Config::from_toml(&template.join(config::CONFIG_NAME))?;

    //Initialize Root
    make_origin_and_move_into(output)?;

    //Initialize Rokit as the package manager
    run_command("rokit", ["init"])?;

    template_config.rokit_tools.iter().for_each(|tool| {
        run_command("rokit", ["add", tool]).expect(format!("Failed to add {} as a tool to rokit", tool).as_str());
    });
    let mut contains_rojo = false;
    if template_config.rokit_tools.contains(&"rojo".to_string()) {
        contains_rojo = true;
        initialize_empty_rojo()?;
    }
    let mut contains_wally = false;
    if template_config.rokit_tools.contains(&"wally".to_string()) {
        contains_wally = true;
        initialize_wally(&template_config.wally)?;
    }
    copy_dir_all(template, ".", match output.file_name() {
        Some(x) => x.to_str().unwrap(),
        None => return Err(Error::new(io::ErrorKind::NotFound, "Can't find file name of path"))
    })?;

    if contains_rojo && contains_wally {
        run_wally_type_handling()?;
    }


    Ok(())
}