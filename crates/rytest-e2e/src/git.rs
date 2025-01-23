
use std::{path::Path, process::Command};

use anyhow::Result;


pub(crate) fn clone(repository: &str, dest: &Path) -> Result<(), anyhow::Error> {
    let mut command = Command::new("git");

    let dest_git = dest.join(".git");
    if !dest_git.exists() {
        let dest_str = match dest.to_str() {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Invalid path: {:?}", dest)),
        };

        println!("Cloning {} to {}", repository, dest_str);

        let mut child = command.args(&["clone", repository, dest_str]).spawn()?;
        child.wait()?;
    } else {
        println!("Skipping clone of {}, already exists.", repository);
    }

    Ok(())
}
