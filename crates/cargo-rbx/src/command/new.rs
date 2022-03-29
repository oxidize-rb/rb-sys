use std::{io::Write, path::PathBuf, str::FromStr};

use crate::CommandExecute;

/// Create a new extension crate
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct New {
    /// The name of the extension
    name: String,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
}

impl CommandExecute for New {
    #[tracing::instrument(level = "error", skip(self))]
    fn execute(self) -> eyre::Result<()> {
        let path = PathBuf::from_str(&format!("{}/", self.name)).unwrap();
        create_crate_template(path, &self.name)
    }
}

#[tracing::instrument(skip_all, fields(path, name))]
pub(crate) fn create_crate_template(path: PathBuf, name: &str) -> eyre::Result<()> {
    create_directory_structure(&path)?;
    create_cargo_toml(&path, name)?;
    create_dotcargo_config(&path, name)?;
    create_lib_rs(&path, name)?;
    create_git_ignore(&path, name)?;

    Ok(())
}

fn create_directory_structure(path: &PathBuf) -> Result<(), std::io::Error> {
    let mut src_dir = path.clone();

    src_dir.push("src");
    std::fs::create_dir_all(&src_dir)?;

    src_dir.pop();
    src_dir.push(".cargo");
    std::fs::create_dir_all(&src_dir)?;

    std::fs::create_dir_all(&src_dir)
}

fn create_cargo_toml(path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    let mut filename = path.clone();

    filename.push("Cargo.toml");
    let mut file = std::fs::File::create(filename)?;

    file.write_all(&format!(include_str!("../templates/cargo_toml"), name = name).as_bytes())?;

    Ok(())
}

fn create_dotcargo_config(path: &PathBuf, _name: &str) -> Result<(), std::io::Error> {
    let mut filename = path.clone();

    filename.push(".cargo");
    filename.push("config");
    let mut file = std::fs::File::create(filename)?;

    file.write_all(include_bytes!("../templates/cargo_config"))?;

    Ok(())
}

fn create_lib_rs(path: &PathBuf, name: &str) -> Result<(), std::io::Error> {
    let mut filename = path.clone();

    filename.push("src");
    filename.push("lib.rs");
    let mut file = std::fs::File::create(filename)?;

    file.write_all(&format!(include_str!("../templates/lib_rs"), name = name).as_bytes())?;

    Ok(())
}

fn create_git_ignore(path: &PathBuf, _name: &str) -> Result<(), std::io::Error> {
    let mut filename = path.clone();

    filename.push(".gitignore");
    let mut file = std::fs::File::create(filename)?;

    file.write_all(include_bytes!("../templates/gitignore"))?;

    Ok(())
}
