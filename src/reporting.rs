//! Error reporting.

use std::{fmt::Display, path::Path};

pub use anyhow::{Result, anyhow};
pub use log::*;

pub trait ResultNote<R> {
    fn with_path_action<P: AsRef<Path>>(self, action: &str, path: P) -> Result<R>;
}

impl<R, E> ResultNote<R> for Result<R, E>
where
    E: Into<anyhow::Error> + Display,
{
    fn with_path_action<P: AsRef<Path>>(self, action: &str, path: P) -> Result<R> {
        self.map_err(|e| anyhow!("{} {}: {}", action, path.as_ref().display(), e))
    }
}
