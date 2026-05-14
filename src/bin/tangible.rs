//! `tangible` CLI entry point.
//!
//! See the [`tangible::cli`] module for the parsed-arguments shape and runtime entry point.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = tangible::cli::Args::parse();
    tangible::cli::run(args, &tangible::io::SystemFs)
}
