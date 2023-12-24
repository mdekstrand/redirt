//! Compute the difference between source and target trees.
//!
//! This module provides a tree-diffing algorihtm to compute the difference between
//! source and target trees.

use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use crate::walk::{WalkBuilder, WalkEntry, Walker};

/// Set up a tree-diff operation.
#[derive(Debug, Clone)]
pub struct TreeDiffBuilder {
    src: WalkBuilder,
    tgt: WalkBuilder,
    check_content: bool,
}

/// Iterater over differences between two trees.
pub struct TreeDiff {
    src: Walker,
    tgt: Walker,
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

impl TreeDiffBuilder {
    pub fn new<SP: AsRef<Path>, TP: AsRef<Path>>(src: SP, tgt: TP) -> TreeDiffBuilder {
        TreeDiffBuilder {
            src: WalkBuilder::for_directory(src),
            tgt: WalkBuilder::for_directory(tgt),
            check_content: true,
        }
    }

    /// Configure whether to follow symlinks (off by default).
    pub fn follow_symlinks(&mut self, follow: bool) -> &mut TreeDiffBuilder {
        self.src.follow_symlinks(follow);
        self.tgt.follow_symlinks(follow);
        self
    }

    /// Configure whether to include hidden files (on by default).
    pub fn include_hidden(&mut self, include: bool) -> &mut TreeDiffBuilder {
        self.src.include_hidden(include);
        self.tgt.include_hidden(include);
        self
    }

    /// Control whether to check content in comparing files (on by default).
    pub fn check_content(&mut self, check: bool) -> &mut TreeDiffBuilder {
        self.check_content = check;
        self
    }

    /// Run the diff.
    pub fn run(self) -> TreeDiff {
        TreeDiff {
            src: self.src.walk(),
            tgt: self.tgt.walk(),
            s_cur: None,
            t_cur: None,
            check_content: self.check_content,
        }
    }
}

impl TreeDiff {
    fn find_next(&mut self) -> io::Result<Option<DiffEntry>> {
        if self.s_cur.is_none() {
            self.s_cur = self.src.next().transpose()?;
        }
        if self.t_cur.is_none() {
            self.t_cur = self.tgt.next().transpose()?;
        }

        let spath = self.s_cur.as_ref().map(|w| w.path().to_owned());
        let tpath = self.t_cur.as_ref().map(|w| w.path().to_owned());

        let (tgt, src) = match (spath, tpath) {
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

        let res = match (tgt, src) {
            (None, None) => None,
            (Some(src), None) => Some(DiffEntry::Added { src }),
            (None, Some(tgt)) => Some(DiffEntry::Removed { tgt }),
            (Some(tgt), Some(src)) => {
                let ch_type = src.file_type() != tgt.file_type();
                let ch_mtime = src.mtime() != tgt.mtime();
                let ch_size = src.size() != tgt.size();
                if ch_type || ch_mtime || ch_size {
                    Some(DiffEntry::Modified {
                        src,
                        tgt,
                        ch_type,
                        ch_mtime,
                        ch_size,
                        ch_content: false,
                    })
                } else if self.check_content && !files_are_identical(src.path(), tgt.path())? {
                    Some(DiffEntry::Modified {
                        src,
                        tgt,
                        ch_type: false,
                        ch_mtime: false,
                        ch_size: false,
                        ch_content: true,
                    })
                } else {
                    Some(DiffEntry::Present { src, tgt })
                }
            }
        };
        Ok(res)
    }
}

impl Iterator for TreeDiff {
    type Item = io::Result<DiffEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.find_next().transpose()
    }
}

fn files_are_identical(f1: &Path, f2: &Path) -> io::Result<bool> {
    let r1 = File::open(f1)?;
    let r1 = BufReader::new(r1);
    let r2 = File::open(f2)?;
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
