//! Walking file systems (and eventually other things).
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::path::StripPrefixError;

use thiserror::Error;

mod fswalk;
mod options;

pub use fswalk::walk_fs;
pub use options::WalkOptions;

#[derive(Error, Debug)]
pub enum WalkError {
    #[error("FS walk failed: {0}")]
    Ignore(#[from] ignore::Error),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("invalid path result: {0}")]
    PathPrefix(#[from] StripPrefixError),
}

/// Single result entry in a tree-walk.
#[derive(Clone)]
pub struct WalkEntry {
    path: PathBuf,
    meta: Option<fs::Metadata>,
}

impl WalkEntry {
    pub fn path(&self) -> &Path {
        return &self.path;
    }

    pub fn metadata(&self) -> Option<&fs::Metadata> {
        self.meta.as_ref()
    }

    pub fn file_type(&self) -> Option<fs::FileType> {
        self.meta.as_ref().map(|m| m.file_type())
    }

    pub fn is_directory(&self) -> bool {
        self.metadata().unwrap().is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.metadata().unwrap().is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.metadata().unwrap().is_symlink()
    }
}

/// Interface for tree-walking.
pub trait TreeWalk: Iterator<Item = Result<WalkEntry, WalkError>> {
    /// Get the path to the root of this walk.
    fn root(&self) -> &Path;
}
