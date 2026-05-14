//! Read a token spec from `tokens.json` (relative to the working directory) and write CSS to
//! `tokens.css`. Run with `cargo run --example from_json`.
//!
//! Uses `std::fs` directly for brevity; production code in this crate goes through
//! [`tangible::io::FileOps`] so it can be tested without touching disk.

// Examples illustrate library usage, not internal conventions — bypass the FileOps boundary.
#![allow(clippy::disallowed_methods)]

use std::error::Error;
use std::fs;

use tangible::{Renderer, Spec};

fn main() -> Result<(), Box<dyn Error>> {
    let json = fs::read_to_string("tokens.json")?;
    let spec: Spec = serde_json::from_str(&json)?;
    let manifest = Renderer::new().render(&spec)?;
    fs::write("tokens.css", manifest.to_string())?;
    println!("Wrote tokens.css ({} bytes)", manifest.as_str().len());
    Ok(())
}
