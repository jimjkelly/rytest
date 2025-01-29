use std::env::current_dir;
use std::path::PathBuf;

pub fn get(segments: Vec<&str>) -> Result<PathBuf, std::io::Error> {
    let mut dir = current_dir()?;
    dir.push(".rytest");
    dir.push("e2e");
    segments.iter().for_each(|segment| dir.push(segment));

    if !dir.exists() {
        println!("Creating directory: {:?}", dir);
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}
