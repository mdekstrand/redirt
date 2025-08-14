//! Compute the difference between source and target trees.
//!
//! This module provides a tree-diffing algorihtm to compute the difference between
//! source and target trees.

use std::{
    fs::File,
    io::{self, BufReader, Read},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use log::*;

use crate::walk::{WalkEntry, WalkError};

/// Iterater over differences between two trees.
pub struct TreeDiff {
    // src_root: PathBuf,
    src: Box<dyn Iterator<Item = Result<WalkEntry, WalkError>>>,
    // tgt_root: PathBuf,
    tgt: Box<dyn Iterator<Item = Result<WalkEntry, WalkError>>>,
    s_cur: Option<WalkEntry>,
    t_cur: Option<WalkEntry>,
    check_content: bool,
}

/// Entry representing a difference between trees.
pub enum DiffEntry {
    /// A file is present in both trees.
    Present { src: WalkEntry, tgt: WalkEntry },
    /// A file has been added in the source tree.
    Added { src: WalkEntry },
    /// A file has been removed in the source tree.
    Removed { tgt: WalkEntry },
    /// A file has been modified in the source tree.
    ///
    /// If the diff is configured not to check content (see  [TreeDiffBuilder::check_content]),
    /// `ch_content` will never be `true`, and modified files with identical sizes and times
    /// may appear as [DiffEntry::Present].  This field is also only set if other file attributes
    /// are unchanged — if the times or dates change, we don't bother checking content.
    Modified {
        src: WalkEntry,
        tgt: WalkEntry,
        ch_type: bool,
        ch_mtime: bool,
        ch_size: bool,
        ch_content: bool,
    },
}

pub fn diff_walkers<Src, Dst>(src: Src, dst: Dst) -> TreeDiff
where
    Src: Iterator<Item = Result<WalkEntry, WalkError>> + 'static,
    Dst: Iterator<Item = Result<WalkEntry, WalkError>> + 'static,
{
    TreeDiff {
        src: Box::new(src),
        tgt: Box::new(dst),
        s_cur: None,
        t_cur: None,
        check_content: false,
    }
}

impl TreeDiff {
    fn find_next(&mut self) -> Result<Option<DiffEntry>, WalkError> {
        if self.s_cur.is_none() {
            self.s_cur = self.src.next().transpose()?;
        }
        if self.t_cur.is_none() {
            self.t_cur = self.tgt.next().transpose()?;
        }

        let spath = self.s_cur.as_ref().map(|w| w.path().to_owned());
        let tpath = self.t_cur.as_ref().map(|w| w.path().to_owned());

        let (src, tgt) = match (spath, tpath) {
            (None, None) => (None, None),
            (None, Some(_)) => (None, self.t_cur.take()),
            (Some(_), None) => (self.s_cur.take(), None),
            (Some(src), Some(tgt)) => {
                if src < tgt {
                    (self.s_cur.take(), None)
                } else if tgt < src {
                    (None, self.t_cur.take())
                } else {
                    (self.s_cur.take(), self.t_cur.take())
                }
            }
        };

        let res = match (src, tgt) {
            (None, None) => None,
            (Some(src), None) => Some(DiffEntry::Added { src }),
            (None, Some(tgt)) => Some(DiffEntry::Removed { tgt }),
            (Some(src), Some(tgt)) => {
                let src_meta = src.metadata().unwrap();
                let tgt_meta = tgt.metadata().unwrap();
                let ch_type = src_meta.file_type() != tgt_meta.file_type();
                let ch_mtime = !src_meta.file_type().is_dir()
                    && src_meta.modified()? != tgt_meta.modified()?;
                let ch_size = !src_meta.file_type().is_dir() && src_meta.size() != tgt_meta.size();
                if ch_type || ch_mtime || ch_size {
                    Some(DiffEntry::Modified {
                        src,
                        tgt,
                        ch_type,
                        ch_mtime,
                        ch_size,
                        ch_content: false,
                    })
                // } else if self.check_content
                //     && !src_meta.file_type().is_dir()
                //     && !files_are_identical(
                //         &self.src_root.join(src.path()),
                //         &self.tgt_root.join(tgt.path()),
                //     )?
                // {
                //     Some(DiffEntry::Modified {
                //         src,
                //         tgt,
                //         ch_type: false,
                //         ch_mtime: false,
                //         ch_size: false,
                //         ch_content: true,
                //     })
                } else {
                    Some(DiffEntry::Present { src, tgt })
                }
            }
        };
        Ok(res)
    }
}

impl Iterator for TreeDiff {
    type Item = Result<DiffEntry, WalkError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.find_next().transpose()
    }
}

fn files_are_identical(f1: &Path, f2: &Path) -> io::Result<bool> {
    debug!(
        "{} and {} identical, checking content",
        f1.display(),
        f2.display()
    );
    let r1 = File::open(f1).map_err(|e| {
        error!("{}: cannot open: {}", f1.display(), e);
        e
    })?;
    let r1 = BufReader::new(r1);
    let r2 = File::open(f2).map_err(|e| {
        error!("{}: cannot open: {}", f1.display(), e);
        e
    })?;
    let r2 = BufReader::new(r2);

    let mut bi1 = r1.bytes();
    let mut bi2 = r2.bytes();

    loop {
        let b1 = bi1.next().transpose()?;
        let b2 = bi2.next().transpose()?;
        match (b1, b2) {
            (Some(b1), Some(b2)) if b1 == b2 => (),
            (None, None) => break,
            _ => return Ok(false),
        }
    }

    Ok(true)
}
