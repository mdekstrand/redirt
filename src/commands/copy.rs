//! directory listing command

use std::path::PathBuf;
use std::{fs, io};

use clap::Args;
use log::*;

use crate::{
    diff::{diff_walkers, DiffEntry},
    walk::{walk_fs, WalkOptions},
};
use anyhow::anyhow;

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

        self.ensure_dest()?;

        let src_walk = walk_fs(&self.src, &self.traverse);
        let dst_walk = walk_fs(&self.dst, &self.traverse);
        let diff = diff_walkers(src_walk, dst_walk);
        let mut n_files = 0;
        let mut n_dirs = 0;
        for entry in diff {
            let entry = entry?;
            match entry {
                DiffEntry::Present { src, .. } => {
                    debug!("file {} already exists", src.path().display());
                }
                DiffEntry::Added { src } if src.is_directory() => {
                    debug!("creating directory {}", src.path().display());
                    let path = self.dst.join(src.path());
                    fs::create_dir(path)?;
                    n_dirs += 1;
                }
                DiffEntry::Added { src } if src.is_file() => {
                    debug!("copying file {}", src.path().display());
                    let src_path = self.src.join(src.path());
                    let dst_path = self.dst.join(src.path());
                    fs::copy(&src_path, &dst_path)?;
                    n_files += 1
                }
                DiffEntry::Added { src } => {
                    error!(
                        "{}: unsupported file type {:?}",
                        src.path().display(),
                        src.file_type()
                    );
                    return Err(anyhow!("unsupported file"));
                }
                DiffEntry::Removed { tgt } => {
                    debug!("destination {} not in source", tgt.path().display());
                }
                DiffEntry::Modified {
                    src, tgt, ch_type, ..
                } => {
                    let dst_path = self.dst.join(tgt.path());
                    if ch_type {
                        if tgt.is_directory() {
                            debug!("removing destination directory {}", tgt.path().display());
                            fs::remove_dir_all(&dst_path)?;
                        } else {
                            debug!("removing destination file {}", tgt.path().display());
                            fs::remove_file(&dst_path)?;
                        }
                        if src.is_directory() {
                            fs::create_dir(&dst_path)?;
                            n_dirs += 1;
                        }
                    }

                    if !src.is_file() {
                        error!(
                            "unexpected or unsupported file type {:?}",
                            src.metadata().map(|m| m.file_type())
                        );
                        return Err(anyhow!("unsupported file type"));
                    }

                    let src_path = self.src.join(src.path());
                    fs::copy(&src_path, &dst_path)?;
                    n_files += 1;
                }
            }
        }
        info!("copied {} files and {} directories", n_files, n_dirs);
        Ok(())
    }
}

impl CopyCmd {
    fn ensure_dest(&self) -> io::Result<()> {
        match fs::metadata(&self.dst) {
            Ok(m) if m.is_dir() => {
                debug!("directory {} exists", self.dst.display());
                Ok(())
            }
            Ok(_m) => {
                error!("{} exists but is not a directory", self.dst.display());
                Err(io::Error::new(
                    io::ErrorKind::NotADirectory,
                    "destination file not a directory",
                ))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                debug!("creating destination directory {}", self.dst.display());
                fs::create_dir(&self.dst)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
