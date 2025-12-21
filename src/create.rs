use std::{ffi::OsStr, fs::{self, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}, process::{Command, Output}};

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

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn file(file_name: PathBuf, file_content: &str) -> std::io::Result<()> {
	let mut file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(file_name)?;

	file.write_all(file_content.as_bytes())?;
	file.flush()?;
	Ok(())
}

pub fn project(output: PathBuf, template: PathBuf) -> std::io::Result<()> {
	//Initialize Rokit as the package manager
	run_command("rokit", ["init"])?;
	

	Ok(())
}