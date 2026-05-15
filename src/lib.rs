//! Design tokens as data — colors, type, space, shadows, gradients, and contrast — rendered to CSS.
//!
//! `tangible` turns a structured token specification into a CSS custom-property sheet you can drop
//! into any project. It is the library that powers the `tangible` command-line tool, but every
//! step in the pipeline (parsing, resolving palettes, picking inks, emitting CSS) is available
//! programmatically for embedding in build scripts, generators, or design-system tooling.
//!
//! # Themes
//!
//! Five sample themes, each rendered from a single JSON spec. Source specs and sample HTML are
//! in the [`samples/`](https://github.com/esmevane/tangible/tree/main/samples) directory of the
//! repository.
//!
//! ### Prose
//!
//! *Editorial light · Literata · blue + amber on warm neutrals*
//!
//! ![prose](https://raw.githubusercontent.com/esmevane/tangible/main/samples/prose.png)
//!
//! ### Starliner
//!
//! *Sleek dark · Sora · violet + emerald*
//!
//! ![starliner](https://raw.githubusercontent.com/esmevane/tangible/main/samples/starliner.png)
//!
//! ### Tropical
//!
//! *Vivid · Bricolage Grotesque · coral + teal*
//!
//! ![tropical](https://raw.githubusercontent.com/esmevane/tangible/main/samples/tropical.png)
//!
//! ### Candy
//!
//! *Sweet light · Caprasimo · pink + lime on cream*
//!
//! ![candy](https://raw.githubusercontent.com/esmevane/tangible/main/samples/candy.png)
//!
//! ### Cosmic
//!
//! *Deep space · Audiowide · magenta + cyan on ink*
//!
//! ![cosmic](https://raw.githubusercontent.com/esmevane/tangible/main/samples/cosmic.png)
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
