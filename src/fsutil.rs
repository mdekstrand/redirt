//! File system utility functions.
use std::fs;
use std::io;
use std::path::Path;

use log::*;

/// Stat a path, cleanly returning `None` if it does not exist.
pub fn stat<P: AsRef<Path>>(path: P) -> io::Result<Option<fs::Metadata>> {
    let path = path.as_ref();
    match fs::metadata(path) {
        Ok(m) => Ok(Some(m)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => {
            error!("{}: stat failed: {}", path.display(), e);
            Err(e)
        }
    }
}
