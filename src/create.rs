use std::{
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::{Command, Output},
};

use anyhow::{bail, Context, Result};

use crate::{
    config::{self, Pesde, Wally},
    log_step,
};

pub fn run_command<T>(command: &str, args: T, working_dir: &Path) -> Result<Output>
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

    let output = Command::new(command)
        .args(&args_vec)
        .current_dir(working_dir)
        .output()
        .with_context(|| format!("Failed to execute `{display}`"))?;

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

    bail!("Command failed: `{display}`\n{details}");
}

/// Recursively copies a directory, replacing `{{project_name}}` in text files.
/// Files are detected as text by attempting UTF-8 decode; failures are copied as binary.
fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    project_name: &str,
) -> Result<()> {
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
            if let Ok(contents) = fs::read_to_string(&src_path) {
                let rendered = contents.replace("{{project_name}}", project_name);
                fs::write(&dst_path, rendered)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}

pub fn write_file(file_name: &Path, file_content: &str) -> Result<()> {
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

fn initialize_empty_rojo(working_dir: &Path) -> Result<()> {
    log_step("Initializing empty Rojo project");
    run_command("rojo", ["init", "--kind", "model"], working_dir)?;
    // Removes the only created file to just have an empty source
    let path_buf = working_dir.join("src").join("init.luau");
    fs::remove_file(&path_buf)?;
    Ok(())
}

fn initialize_wally(wally_dependencies: Option<&Wally>, working_dir: &Path) -> Result<()> {
    log_step("Initializing Wally");
    run_command("wally", ["init"], working_dir)?;

    if let Some(wally_dependencies) = wally_dependencies {
        wally_dependencies.write_to_wally(&working_dir.join("wally.toml"))?;
    } else {
        log_step("No Wally dependencies in template; keeping generated wally.toml");
    }

    Ok(())
}

fn initialize_pesde(pesde_dependencies: Option<&Pesde>, working_dir: &Path) -> Result<()> {
    log_step("Initializing pesde");
    run_command("pesde", ["init"], working_dir)?;

    if let Some(pesde_deps) = pesde_dependencies {
        pesde_deps.write_to_pesde(&working_dir.join("pesde.toml"))?;
    } else {
        log_step("No pesde dependencies in template; keeping generated pesde.toml");
    }

    Ok(())
}

pub fn run_pesde_install(working_dir: &Path) -> Result<()> {
    log_step("Installing pesde packages");
    run_command("pesde", ["install"], working_dir)?;
    Ok(())
}

pub fn run_wally_type_handling(working_dir: &Path) -> Result<()> {
    log_step("Generating Wally package types");
    run_command("wally", ["install"], working_dir)?;

    // Rojo sourcemap fails if a referenced $path directory does not exist yet.
    let packages_dir = working_dir.join("Packages");
    if !packages_dir.try_exists()? {
        log_step("Creating empty Packages directory");
        fs::create_dir_all(&packages_dir)?;
    }

    run_command(
        "rojo",
        ["sourcemap", "default.project.json", "--output", "sourcemap.json"],
        working_dir,
    )?;
    run_command(
        "wally-package-types",
        ["--sourcemap", "sourcemap.json", "Packages/"],
        working_dir,
    )?;
    Ok(())
}

pub fn project(output: &Path, template: &Path) -> Result<()> {
    log_step("Loading template configuration");
    let template_config = config::Config::from_toml(&template.join(config::CONFIG_NAME))?;

    log_step(format!("Creating project folder at {}", output.display()).as_str());
    fs::create_dir(output).with_context(|| format!("Failed to create directory {}", output.display()))?;

    let result = setup_project(output, template, &template_config);
    if result.is_err() {
        log_step("Project creation failed, cleaning up");
        let _ = fs::remove_dir_all(output);
    }
    result
}

fn setup_project(output: &Path, template: &Path, template_config: &config::Config) -> Result<()> {
    // Initialize Rokit as the package manager
    log_step("Initializing Rokit");
    run_command("rokit", ["init"], output)?;

    for spec in template_config.rokit.specs() {
        run_command("rokit", ["add", spec], output)?;
    }

    let contains_rojo = template_config.rokit.has_tool("rojo");
    if contains_rojo {
        initialize_empty_rojo(output)?;
    }

    let contains_wally = template_config.rokit.has_tool("wally");
    let has_wally_dependencies = template_config
        .wally
        .as_ref()
        .map(|w| w.has_dependencies())
        .unwrap_or(false);
    if contains_wally {
        initialize_wally(template_config.wally.as_ref(), output)?;
    }

    let contains_pesde = template_config.rokit.has_tool("pesde");
    let has_pesde_dependencies = template_config
        .pesde
        .as_ref()
        .map(|p| p.has_dependencies())
        .unwrap_or(false);
    if contains_pesde {
        initialize_pesde(template_config.pesde.as_ref(), output)?;
    }

    let project_name = output
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .context("Cannot determine project name from output path")?;

    log_step("Copying template files");
    copy_dir_all(template, output, &project_name)?;

    if contains_rojo && contains_wally && has_wally_dependencies {
        run_wally_type_handling(output)?;
    } else if contains_rojo && contains_wally {
        log_step("No Wally dependencies configured; skipping package type generation");
    }

    if contains_pesde && has_pesde_dependencies {
        run_pesde_install(output)?;
    }
    let contains_lune = template_config.rokit.has_tool("lune");
    if contains_lune {
        run_command("lune", ["setup"], output)?;
    }
    log_step("Project scaffold complete");
    Ok(())
}
