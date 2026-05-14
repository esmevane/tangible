//! Filesystem boundary for `tangible`'s CLI layer.
//!
//! The library proper never touches the filesystem — it operates on already-parsed [`Spec`]
//! values. The CLI does, and to keep that boundary explicit (and testable) every disk-facing
//! call goes through the [`FileOps`] trait rather than `std::fs` directly. The default
//! implementation, [`SystemFs`], delegates to the standard library; tests and integrators can
//! substitute their own backends (in-memory, dry-run, virtual, …).
//!
//! This module is only available when the `cli` feature is enabled.
//!
//! # Example
//!
//! ```ignore
//! use tangible::cli::{run, Args};
//! use tangible::io::SystemFs;
//!
//! let args = Args { input: "tokens.json".into(), output: None };
//! run(args, &SystemFs)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! [`Spec`]: crate::Spec

use std::path::Path;

use crate::error::FileOpsError;

/// Abstracts the filesystem operations needed by `tangible`'s CLI.
///
/// Implementations decide what a "filesystem" means — disk, memory, a recorder for tests, a
/// dry-run logger, anything that fits the surface. Methods return [`Result<T, FileOpsError>`] so
/// callers can add their own context layer (the CLI uses [`anyhow::Context`](anyhow)).
pub trait FileOps {
    /// Reads the contents of `path` as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns [`FileOpsError`] if the file does not exist or cannot be read.
    fn read_to_string(&self, path: &Path) -> Result<String, FileOpsError>;

    /// Writes `contents` to `path`, replacing any existing file.
    ///
    /// # Errors
    ///
    /// Returns [`FileOpsError`] if the file cannot be written.
    fn write(&self, path: &Path, contents: &[u8]) -> Result<(), FileOpsError>;

    /// Recursively creates `path` and all of its parent components if they do not already exist.
    ///
    /// # Errors
    ///
    /// Returns [`FileOpsError`] if any directory in the path cannot be created.
    fn create_dir_all(&self, path: &Path) -> Result<(), FileOpsError>;
}

/// The default [`FileOps`] implementation, backed by [`std::fs`].
///
/// `SystemFs` is a zero-sized type — pass it by reference (`&SystemFs`) wherever a `FileOps`
/// impl is expected.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemFs;

impl FileOps for SystemFs {
    fn read_to_string(&self, path: &Path) -> Result<String, FileOpsError> {
        #[allow(clippy::disallowed_methods)]
        std::fs::read_to_string(path).map_err(FileOpsError::from)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<(), FileOpsError> {
        #[allow(clippy::disallowed_methods)]
        std::fs::write(path, contents).map_err(FileOpsError::from)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FileOpsError> {
        #[allow(clippy::disallowed_methods)]
        std::fs::create_dir_all(path).map_err(FileOpsError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io;
    use std::path::PathBuf;

    /// Tiny in-memory stub used to confirm the trait is genuinely substitutable.
    #[derive(Default)]
    struct MemoryFs {
        files: RefCell<HashMap<PathBuf, Vec<u8>>>,
    }

    impl FileOps for MemoryFs {
        fn read_to_string(&self, path: &Path) -> Result<String, FileOpsError> {
            self.files
                .borrow()
                .get(path)
                .map(|bytes| String::from_utf8(bytes.clone()).expect("non-utf8 in MemoryFs"))
                .ok_or_else(|| {
                    FileOpsError::from(io::Error::new(io::ErrorKind::NotFound, "not found"))
                })
        }

        fn write(&self, path: &Path, contents: &[u8]) -> Result<(), FileOpsError> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), contents.to_vec());
            Ok(())
        }

        fn create_dir_all(&self, _path: &Path) -> Result<(), FileOpsError> {
            Ok(())
        }
    }

    #[test]
    fn memory_fs_round_trips() {
        let fs = MemoryFs::default();
        fs.write(Path::new("a.txt"), b"hello").unwrap();
        assert_eq!(fs.read_to_string(Path::new("a.txt")).unwrap(), "hello");
    }

    #[test]
    fn memory_fs_missing_file_is_not_found() {
        let fs = MemoryFs::default();
        let err = fs.read_to_string(Path::new("missing.txt")).unwrap_err();
        assert_eq!(err.0.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn system_fs_round_trips_a_real_file() {
        let dir = std::env::temp_dir().join("tangible-io-roundtrip");
        SystemFs.create_dir_all(&dir).unwrap();
        let path = dir.join("hello.txt");
        SystemFs.write(&path, b"hi from tangible").unwrap();
        assert_eq!(SystemFs.read_to_string(&path).unwrap(), "hi from tangible");
    }
}
