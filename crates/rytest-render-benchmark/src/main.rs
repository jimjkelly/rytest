/*

This is majorly inspired by uv's `render-benchmarks` subcommand in their `uv-dev` crate.

Thank you to them for the code and inspiration for repo layout!

*/

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use poloto::build;
use resvg::usvg_text_layout::{fontdb, TreeTextToPath};
use serde::Deserialize;
use tagu::prelude::*;

#[derive(Parser)]
#[clap(about = "Renders benchmark output into a pretty image.")]
struct Args {
    /// Path to a JSON output from a `hyperfine` benchmark.
    path: PathBuf,
    /// Title of the plot.
    #[clap(long, short)]
    title: Option<String>,
}

fn render_benchmarks(args: &Args) -> Result<()> {
    let mut results: BenchmarkResults = serde_json::from_slice(&fs_err::read(&args.path)?)?;

    // Replace the command with a shorter name. (The command typically includes the benchmark name,
    // but we assume we're running over a single benchmark here.)
    for result in &mut results.results {
        if result.command.starts_with("target/release/rytest") {
            result.command = "rytest".into();
        } else if result.command.starts_with("pytest") {
            result.command = "pytest".into();
        } else {
            return Err(anyhow!("unknown command: {}", result.command));
        }
    }

    let fontdb = load_fonts();

    render_to_png(
        &plot_benchmark(args.title.as_deref().unwrap_or("Benchmark"), &results)?,
        &args.path.with_extension("png"),
        &fontdb,
    )?;

    Ok(())
}

/// Render a benchmark to an SVG (as a string).
fn plot_benchmark(heading: &str, results: &BenchmarkResults) -> Result<String> {
    let mut data = Vec::new();
    for result in &results.results {
        data.push((result.mean, &result.command));
    }

    let theme = poloto::render::Theme::light();
    let theme = theme.append(tagu::build::raw(
        ".poloto0.poloto_fill{fill: #6340AC !important;}",
    ));
    let theme = theme.append(tagu::build::raw(
        ".poloto_background{fill: white !important;}",
    ));

    Ok(build::bar::gen_simple("", data, [0.0])
        .label((heading, "Time (s)", ""))
        .append_to(poloto::header().append(theme))
        .render_string()?)
}

/// Render an SVG to a PNG file.
fn render_to_png(data: &str, path: &Path, fontdb: &fontdb::Database) -> Result<()> {
    let mut tree = resvg::usvg::Tree::from_str(data, &resvg::usvg::Options::default())?;
    tree.convert_text(fontdb);
    let fit_to = resvg::usvg::FitTo::Width(1600);
    let size = fit_to
        .fit_to(tree.size.to_screen_size())
        .ok_or_else(|| anyhow!("failed to fit to screen size"))?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
    resvg::render(
        &tree,
        fit_to,
        resvg::tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .ok_or_else(|| anyhow!("failed to render"))?;
    fs_err::create_dir_all(path.parent().unwrap())?;
    pixmap.save_png(path)?;
    Ok(())
}

/// Load the system fonts and set the default font families.
fn load_fonts() -> fontdb::Database {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    fontdb.set_serif_family("Times New Roman");
    fontdb.set_sans_serif_family("Arial");
    fontdb.set_cursive_family("Comic Sans MS");
    fontdb.set_fantasy_family("Impact");
    fontdb.set_monospace_family("Courier New");

    fontdb
}

#[derive(Debug, Deserialize)]
struct BenchmarkResults {
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkResult {
    command: String,
    mean: f64,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = render_benchmarks(&args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
