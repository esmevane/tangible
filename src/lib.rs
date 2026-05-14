//! Design tokens as data — colors, type, space, shadows, gradients, and contrast — rendered to CSS.
//!
//! `tangible` turns a structured token specification into a CSS custom-property sheet you can drop
//! into any project. It is the library that powers the `tangible` command-line tool, but every
//! step in the pipeline (parsing, resolving palettes, picking inks, emitting CSS) is available
//! programmatically for embedding in build scripts, generators, or design-system tooling.
//!
//! # At a glance
//!
//! ```no_run
//! use tangible::{Renderer, Spec};
//!
//! let json = std::fs::read_to_string("tokens.json")?;
//! let spec: Spec = serde_json::from_str(&json)?;
//! let manifest = Renderer::new().render(&spec)?;
//! std::fs::write("dist/tokens.css", manifest.to_string())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Modules
//!
//! - [`color`] — color definitions, gradients, interpolation, sampling.
//! - [`contrast`] — WCAG-driven ink selection for legible text on arbitrary backgrounds.
//! - [`spec`] — the [`Spec`] structure and its sub-types (the input format).
//! - [`render`] — the [`Renderer`], its [`Config`], and the resulting [`Manifest`].

#![doc(html_root_url = "https://docs.rs/tangible/0.0.1")]

pub mod color;
pub mod contrast;
pub mod error;
pub mod render;
pub mod spec;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
pub mod io;

pub use crate::error::Error;
pub use crate::render::{Config, Manifest, Renderer};
pub use crate::spec::Spec;

/// A specialized [`Result`] type for `tangible` operations.
pub type Result<T> = std::result::Result<T, Error>;
