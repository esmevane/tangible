//! The crate-wide error type.
//!
//! Every fallible operation in `tangible` returns a [`Result<T, Error>`](crate::Result). The
//! variants below describe each class of failure that callers may need to discriminate when
//! handling errors at the boundary of their application.

use std::fmt;

use thiserror::Error;

/// A filesystem-level error produced by a [`FileOps`](crate::io::FileOps) implementation.
///
/// Wraps a [`std::io::Error`] to keep the IO concern distinct from the token-pipeline errors
/// in [`Error`].
#[derive(Debug)]
pub struct FileOpsError(pub(crate) std::io::Error);

impl fmt::Display for FileOpsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "filesystem error: {}", self.0)
    }
}

impl std::error::Error for FileOpsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<std::io::Error> for FileOpsError {
    fn from(err: std::io::Error) -> Self {
        FileOpsError(err)
    }
}

/// Errors produced by the `tangible` library.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// A semantic, gradient, or ink token referenced a palette that doesn't exist in the spec.
    #[error("unknown palette {palette:?}{context}")]
    UnknownPalette {
        /// The palette name that was referenced.
        palette: String,
        /// Human-readable context describing where the reference appeared.
        context: String,
    },

    /// A token referenced a shade that doesn't exist in `scales.shades`.
    #[error("unknown shade {shade}{context}")]
    UnknownShade {
        /// The shade value that was referenced.
        shade: u32,
        /// Human-readable context describing where the reference appeared.
        context: String,
    },

    /// A string could not be parsed as a CSS color.
    #[error("invalid color {color:?}: {source}")]
    InvalidColor {
        /// The offending input.
        color: String,
        /// The underlying parse error.
        #[source]
        source: csscolorparser::ParseColorError,
    },

    /// A gradient could not be constructed from its stops.
    #[error("invalid gradient {name:?}: {message}")]
    InvalidGradient {
        /// The gradient's name (if any).
        name: String,
        /// Description of the failure.
        message: String,
    },

    /// A gradient stop's position was malformed or out of range.
    #[error("invalid position {0:?}: {1}")]
    InvalidPosition(String, String),

    /// A gradient stop string was malformed.
    #[error("invalid gradient stop {0:?}: {1}")]
    InvalidGradientStop(String, String),

    /// The spec's scales were inconsistent (e.g. a section's array doesn't match `scales.sizes`).
    #[error("scale mismatch: {0}")]
    ScaleMismatch(String),

    /// An IO error occurred during filesystem access.
    #[cfg(feature = "cli")]
    #[error(transparent)]
    Io(#[from] FileOpsError),
}
