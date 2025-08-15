//! directory listing command

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

use clap::Args;
use log::*;

use crate::fsutil::stat;
use crate::walk::{walk_fs, WalkOptions};

use super::Command;

/// List a directory.
#[derive(Debug, Args)]
#[command(name = "copy")]
pub struct CopyCmd {
    #[command(flatten)]
    traverse: WalkOptions,

    /// The source directory.
    #[arg(name = "SRC")]
    src: PathBuf,

    /// The destination directory.
    #[arg(name = "DST")]
    dst: PathBuf,
}

impl Command for CopyCmd {
    fn run(&self) -> anyhow::Result<()> {
        info!("copying directory {:?}", self.src);
        self.create_dir(&self.dst, true)?;

        let src_walk = walk_fs(&self.src, &self.traverse)?;
        let mut n_files = 0;
        let mut n_dirs = 0;
        for entry in src_walk {
            let entry = entry?;
            if entry.is_directory() {
                let path = self.dst.join(entry.path());
                if self.create_dir(&path, false)? {
                    n_dirs += 1;
                }
            } else if entry.is_file() {
                if self.copy_file(entry.path())? {
                    n_files += 1
                }
            } else {
                error!("{}: unsupported file type", entry.path().display());
            }
        }

        info!("copied {} files and {} directories", n_files, n_dirs);
        Ok(())
    }
}

impl CopyCmd {
    fn create_dir(&self, dir: &Path, recursive: bool) -> io::Result<bool> {
        if let Some(m) = stat(dir)? {
            if m.is_dir() {
                debug!("{}: directory already exists", dir.display());
                return Ok(false);
            } else {
                debug!("{}: removing existing path", dir.display());
                fs::remove_file(dir)?;
            }
        };

        debug!("{}: creating directory", dir.display());
        if recursive {
            fs::create_dir_all(dir)?;
        } else {
            fs::create_dir(dir)?;
        }
        Ok(true)
    }

    fn copy_file(&self, path: &Path) -> io::Result<bool> {
        let src_path = self.src.join(path);
        let dst_path = self.dst.join(path);

        let src_meta = fs::metadata(&src_path)?;

        if let Some(m) = stat(&dst_path)? {
            if m.is_dir() {
                warn!("{}: exists but is directory, removing", path.display());
                fs::remove_dir_all(&dst_path)?;
            } else if !m.is_file() {
                warn!("{}: exists but is not file, removing", path.display());
                fs::remove_file(&dst_path)?;
            } else if m.modified()? >= src_meta.modified()? && m.size() == src_meta.size() {
                debug!("{}: exists and is up-to-date", path.display());
                return Ok(false);
            }
        }

        let tmp_path = dst_path.with_extension(".rdt.tmp");
        debug!("{}: copying to temporary file", path.display());
        fs::copy(&src_path, &tmp_path)?;
        debug!("{}: replacing destination file", path.display());
        fs::rename(&tmp_path, &dst_path)?;
        Ok(true)
    }
}
