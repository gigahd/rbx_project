use std::{
    env::set_current_dir,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::{self, Error, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::config::{self, Wally};

fn log_step(message: &str) {
    println!("[rbx_project] {message}");
}

pub fn run_command<T>(command: &str, args: T) -> std::io::Result<Output>
where
    T: IntoIterator,
    T::Item: AsRef<OsStr>,
{
    let args_vec: Vec<OsString> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect();
    let display_args = args_vec
        .iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let display = if display_args.is_empty() {
        command.to_string()
    } else {
        format!("{command} {display_args}")
    };

    log_step(format!("Running `{display}`").as_str());

    let output = Command::new(command).args(&args_vec).output()?;

    if output.status.success() {
        return Ok(output);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if !stderr.is_empty() { stderr } else { stdout };
    let details = if details.is_empty() {
        "No output provided by command".to_string()
    } else {
        details
    };

    Err(Error::new(
        ErrorKind::Other,
        format!("Command failed: `{display}`\n{details}"),
    ))
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
    log_step("Initializing empty Rojo project");
    run_command("rojo", ["init", "--kind", "model"])?;
    //Removes the only created file to just have an empty source
    let path_buf = PathBuf::new().join("src").join("init.luau");
    fs::remove_file(&path_buf)?;
    Ok(())
}

fn initialize_wally(wally_dependencies: Option<&Wally>) -> std::io::Result<()> {
    log_step("Initializing Wally");
    run_command("wally", ["init"])?;

    if let Some(wally_dependencies) = wally_dependencies {
        wally_dependencies.write_to_wally(PathBuf::from("wally.toml"))?;
    } else {
        log_step("No Wally dependencies in template; keeping generated wally.toml");
    }

    Ok(())
}

pub fn run_wally_type_handling() -> std::io::Result<()> {
    log_step("Generating Wally package types");
    run_command("wally", ["install"])?;

    // Rojo sourcemap fails if a referenced $path directory does not exist yet.
    if !Path::new("Packages").try_exists()? {
        log_step("Creating empty Packages directory");
        fs::create_dir_all("Packages")?;
    }

    run_command(
        "rojo",
        ["sourcemap", "default.project.json", "--output", "sourcemap.json"],
    )?;
    run_command(
        "wally-package-types",
        ["--sourcemap", "sourcemap.json", "Packages/"],
    )?;
    Ok(())
}

pub fn project(output: &PathBuf, template: &PathBuf) -> std::io::Result<()> {
    log_step("Loading template configuration");
    let template_config = config::Config::from_toml(&template.join(config::CONFIG_NAME))?;

    //Initialize Root
    log_step(format!("Creating project folder at {}", output.display()).as_str());
    make_origin_and_move_into(output)?;

    //Initialize Rokit as the package manager
    log_step("Initializing Rokit");
    run_command("rokit", ["init"])?;

    for tool in &template_config.rokit_tools {
        run_command("rokit", ["add", tool])?;
    }

    let contains_rojo = template_config.rokit_tools.iter().any(|tool| tool == "rojo");
    if contains_rojo {
        initialize_empty_rojo()?;
    }

    let contains_wally = template_config.rokit_tools.iter().any(|tool| tool == "wally");
    let has_wally_dependencies = template_config
        .wally
        .as_ref()
        .map(|w| !w.shared.is_empty() || !w.server.is_empty())
        .unwrap_or(false);
    if contains_wally {
        initialize_wally(template_config.wally.as_ref())?;
    }

    let project_name = output
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or_else(|| Error::new(io::ErrorKind::NotFound, "Can't find file name of path"))?;

    log_step("Copying template files");
    copy_dir_all(template, ".", &project_name)?;

    if contains_rojo && contains_wally && has_wally_dependencies {
        run_wally_type_handling()?;
    } else if contains_rojo && contains_wally {
        log_step("No Wally dependencies configured; skipping package type generation");
    }

    log_step("Project scaffold complete");
    Ok(())
}


