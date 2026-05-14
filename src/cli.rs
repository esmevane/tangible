//! Command-line interface, gated behind the `cli` feature.
//!
//! The CLI is a thin shell over [`crate::Renderer`]: it reads a JSON spec from a path, renders it
//! into CSS, and writes the result to either a file (`-o`) or stdout.
//!
//! Filesystem access goes through the [`FileOps`](crate::io::FileOps) trait so callers can
//! substitute their own backend. The binary built when the `cli` feature is enabled wires in
//! the default [`SystemFs`](crate::io::SystemFs) implementation.

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::io::FileOps;
use crate::{Renderer, Spec};

/// Render design tokens from a JSON spec into CSS.
#[derive(Parser, Debug)]
#[command(name = "tangible", version, about)]
pub struct Args {
    /// Path to the JSON spec file.
    pub input: PathBuf,

    /// Output file path. If omitted, the CSS is written to stdout.
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
}

/// Execute the CLI with the given arguments, routing every filesystem call through `files`.
///
/// Returns once the manifest has been written (to file or stdout). On any error, the underlying
/// failure is wrapped with [`anyhow::Context`] describing the failing step.
///
/// # Errors
///
/// Returns an error if the input file cannot be read, the spec cannot be parsed, the manifest
/// cannot be rendered, or the output cannot be written.
pub fn run<F: FileOps>(args: Args, files: &F) -> Result<()> {
    let json = files
        .read_to_string(&args.input)
        .with_context(|| format!("reading spec from {}", args.input.display()))?;
    let spec: Spec = serde_json::from_str(&json)
        .with_context(|| format!("parsing spec at {}", args.input.display()))?;
    let manifest = Renderer::new()
        .render(&spec)
        .context("rendering manifest")?;

    if let Some(path) = args.output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                files
                    .create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
        }
        files
            .write(&path, manifest.as_str().as_bytes())
            .with_context(|| format!("writing to {}", path.display()))?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle
            .write_all(manifest.as_str().as_bytes())
            .context("writing to stdout")?;
    }
    Ok(())
}
