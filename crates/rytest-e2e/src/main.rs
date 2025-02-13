/*

rytest-e2e

This tool manages our e2e tests.  It fetches the the configured repositories
and runs the tests in them using the `rytest` binary.  It then generates a
report of the results, returning 0 if the tests pass and 1 if they fail.

*/

use std::path::Path;

use anyhow::Result;
use clap::Parser;
use url::Url;

mod dirs;
mod git;
mod uv;

#[derive(Parser)]
#[clap(about = "Run e2e tests for the rytest project.")]
struct Args {
    repository: String,
    requirements: String,
    command: String,
    directory: Option<String>,
}

fn run(args: &Args) -> Result<()> {
    let repository = &args.repository;
    let requirements = Path::new(&args.requirements);
    let command = &args.command;
    let testdir = args.directory.as_deref();

    let url = Url::parse(repository)?;
    let segments = match url.path_segments() {
        Some(segments) => segments,
        None => return Err(anyhow::anyhow!("Invalid repository URL: {}", repository)),
    };

    let dir = dirs::get(segments.collect::<Vec<&str>>())?;
    let dir = dir.as_path();
    
    println!("Running e2e tests for repository: {} in {}", repository, dir.display());
    git::clone(repository, dir)?;
    uv::install(dir, requirements)?;
    uv::develop(dir)?;
    uv::run(dir, command, testdir)?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
