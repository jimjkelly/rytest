use std::{path::Path, process::Command};

use anyhow::Result;

fn cmd(args: &[&str], path: &Path) -> Result<(), anyhow::Error> {
    let mut command = Command::new("uv");

    let cur_dir = std::env::current_dir()?;

    std::env::set_current_dir(path)?;

    let mut child = match command.args(args).spawn() {
        Ok(c) => c,
        Err(e) => {
            std::env::set_current_dir(cur_dir)?;
            return Err(e.into());
        }
    };

    match child.wait() {
        Ok(_) => (),
        Err(e) => {
            std::env::set_current_dir(cur_dir)?;
            return Err(e.into());
        }
    }

    std::env::set_current_dir(cur_dir)?;

    Ok(())
}

pub(crate) fn install(path: &Path, requirements: &std::path::Path) -> Result<(), anyhow::Error> {
    let requirements_str = requirements
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path: {:?}", requirements))?;

    cmd(&["pip", "install", "-r", requirements_str], path)
}

pub(crate) fn develop(path: &Path) -> Result<(), anyhow::Error> {
    cmd(&["pip", "install", "-e", "."], path)
}

pub(crate) fn run(
    path: &std::path::Path,
    command: &str,
    testdir: Option<&str>,
) -> Result<(), anyhow::Error> {
    let mut args = vec!["run", command];

    if let Some(testdir) = testdir {
        args.push(testdir);
    }

    cmd(&args, path)
}
