//! Walk file trees.

use std::collections::VecDeque;
use std::fs::{read_dir, DirEntry, FileType};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::SystemTime;
use std::{fs, io, thread};

use log::*;

use crate::fsutils::is_hidden;
use crate::stack::Stack;

const DEFAULT_BUFFER: usize = 1000;

#[derive(Debug, PartialEq, Eq)]
pub enum DirPosition {
    First,
    Last,
    Never,
}

/// Set up a tree-walker.
pub struct WalkBuilder {
    root: PathBuf,
    buf_size: usize,
    follow_symlinks: bool,
    include_hidden: bool,
    dirs: DirPosition,
}

/// Implementation of tree-walking.
pub struct Walker {
    channel: Receiver<io::Result<WalkEntry>>,
}

/// Single result entry in a tree-walk.
pub struct WalkEntry {
    path: PathBuf,
    file_type: FileType,
    size: u64,
    mtime: Option<SystemTime>,
    ctime: Option<SystemTime>,
}

struct WWMemory {
    config: WalkBuilder,
    work: Stack<WEntry>,
}

/// Entry in the walk worker stack.
enum WEntry {
    /// Scan a directory.
    ScanDir(PathBuf),
    Emit(WalkEntry),
    Process {
        dir: PathBuf,
        entries: VecDeque<DirEntry>,
    },
}

/// Single task in the walk worker.
enum WTask {
    ScanDir(PathBuf),
    Emit(WalkEntry),
    Process { dir: PathBuf, entry: DirEntry },
    Noop,
    Finished,
}

impl WalkBuilder {
    /// Create a walk builder for a root directory.
    pub fn for_directory<P: AsRef<Path>>(root: P) -> WalkBuilder {
        WalkBuilder {
            root: root.as_ref().to_owned(),
            buf_size: DEFAULT_BUFFER,
            follow_symlinks: false,
            include_hidden: true,
            dirs: DirPosition::First,
        }
    }

    /// Follow symbolic links (off by default).
    pub fn follow_symlinks(&mut self, follow: bool) -> &mut WalkBuilder {
        self.follow_symlinks = follow;
        self
    }

    /// Include hidden files (on by default).
    ///
    /// When `false`, this omits hidden files (beginning with `.` on Unix, and
    /// the HIDDEN attribute on Windows).  While it is on by default in the API,
    /// it is off by default in the CLI.
    pub fn include_hidden(&mut self, include: bool) -> &mut WalkBuilder {
        self.include_hidden = include;
        self
    }

    /// Specify when directories are listed (first by default).
    pub fn dir_position(&mut self, pos: DirPosition) -> &mut WalkBuilder {
        self.dirs = pos;
        self
    }

    /// Start walking the directory.
    pub fn walk(self) -> Walker {
        let (send, recv) = sync_channel(self.buf_size);
        let _handle = thread::spawn(move || self.walk_worker(send));
        Walker { channel: recv }
    }

    fn walk_worker(self, chan: SyncSender<io::Result<WalkEntry>>) -> usize {
        debug!("starting directory scan at {:?}", self.root);
        let mut count = 0;
        let mut stack = WWMemory::new(self);
        stack.work.push(WEntry::ScanDir(PathBuf::new()));
        loop {
            match stack.pump(&chan) {
                Ok(n) if n < 0 => return count,
                Ok(n) => count += n as usize,
                Err(e) => {
                    // if there is an error, we send the error to the channel, and let the thread succeed
                    chan.send(Err(e)).expect("receiver disconnected");
                    return count;
                }
            }
        }
    }
}

impl WWMemory {
    fn new(config: WalkBuilder) -> WWMemory {
        WWMemory {
            config,
            work: Stack::new(),
        }
    }

    fn pump(&mut self, chan: &SyncSender<io::Result<WalkEntry>>) -> io::Result<i32> {
        match self.next_task() {
            WTask::ScanDir(dir) => {
                let entries = self.scan_dir(&dir)?;
                self.work.push(WEntry::Process { dir, entries });
                Ok(0)
            }

            WTask::Emit(w) => {
                chan.send(Ok(w)).expect("receiver hung up");
                Ok(1)
            }

            WTask::Process { dir, entry } => {
                if let Some(w) = self.scan_entry(dir, entry)? {
                    chan.send(Ok(w)).expect("receiver hung up");
                    Ok(1)
                } else {
                    Ok(0)
                }
            }

            WTask::Noop => Ok(0),

            WTask::Finished => Ok(-1),
        }
    }

    fn next_task(&mut self) -> WTask {
        match self.work.pop() {
            Some(WEntry::ScanDir(dir)) => WTask::ScanDir(dir),
            Some(WEntry::Emit(w)) => WTask::Emit(w),
            Some(WEntry::Process { dir, mut entries }) => {
                if let Some(entry) = entries.pop_front() {
                    self.work.push(WEntry::Process {
                        dir: dir.clone(),
                        entries,
                    });
                    WTask::Process { dir, entry }
                } else {
                    WTask::Noop
                }
            }
            None => WTask::Finished,
        }
    }

    fn scan_dir<P: AsRef<Path>>(&mut self, path: P) -> io::Result<VecDeque<DirEntry>> {
        debug!(
            "{}: scanning dir {}",
            self.config.root.display(),
            path.as_ref().display()
        );
        let dir = self.config.root.join(path);
        trace!("scanning {:?}", dir);
        let dir = read_dir(&dir)?;
        let mut entries = Vec::with_capacity(100);

        for ent in dir {
            let ent = ent?;
            if self.config.include_hidden || !is_hidden(&ent)? {
                entries.push(ent);
            }
        }
        entries.sort_by_key(|e| e.file_name());
        Ok(entries.into())
    }

    fn scan_entry(&mut self, dir: PathBuf, entry: DirEntry) -> io::Result<Option<WalkEntry>> {
        debug!(
            "{} / {}: scanning entry {}",
            self.config.root.display(),
            dir.display(),
            entry.file_name().to_string_lossy()
        );
        let path = dir.join(entry.file_name());
        let meta = if self.config.follow_symlinks {
            fs::metadata(self.config.root.join(&path))?
        } else {
            entry.metadata()?
        };
        let file_type = meta.file_type();

        let mut w = Some(WalkEntry {
            path,
            file_type,
            size: meta.len(),
            mtime: meta.modified().ok(),
            ctime: meta.created().ok(),
        });

        if file_type.is_dir() {
            let path = w.as_ref().unwrap().path.clone();

            // if the dirst go last, then we want to push the dir's entry before
            // the directory, so it gets emitted after processing it.
            if self.config.dirs == DirPosition::Last {
                self.work.push(WEntry::Emit(w.take().unwrap()));
            }

            self.work.push(WEntry::ScanDir(path));

            // if never, then clear out the entry.
            if self.config.dirs == DirPosition::Never {
                w = None;
            }
        }

        Ok(w)
    }
}

impl Iterator for Walker {
    type Item = io::Result<WalkEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.channel.recv().ok()
    }
}

impl WalkEntry {
    /// Get the path of this entry (relative to the root).
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}
