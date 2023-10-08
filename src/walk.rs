//! Walk file trees.

use std::collections::VecDeque;
use std::fs::{read_dir, DirEntry, FileType};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::SystemTime;
use std::{fs, io, thread};

use log::*;

const DEFAULT_BUFFER: usize = 1000;

/// Set up a tree-walker.
pub struct WalkBuilder {
    root: PathBuf,
    buf_size: usize,
    follow_symlinks: bool,
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
    stack: Option<Box<QueueNode>>,
}

/// Entry in the walk worker queue.
enum QueueEntry {
    Scan(PathBuf),
    Process {
        dir: PathBuf,
        entries: VecDeque<DirEntry>,
    },
}

struct QueueNode {
    entry: QueueEntry,
    next: Option<Box<QueueNode>>,
}

/// Single task in the walk worker.
enum QueueTask {
    Scan(PathBuf),
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
        }
    }

    /// Follow symbolic links.
    pub fn follow_symlinks(self) -> WalkBuilder {
        WalkBuilder {
            follow_symlinks: true,
            ..self
        }
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
        stack.push_dir(PathBuf::new());
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
            stack: None,
        }
    }

    fn push_dir(&mut self, path: PathBuf) {
        let next = self.stack.take();
        self.stack = Some(Box::new(QueueNode {
            entry: QueueEntry::Scan(path),
            next,
        }));
    }

    fn push_dir_queue(&mut self, dir: PathBuf, entries: VecDeque<DirEntry>) {
        let next = self.stack.take();
        self.stack = Some(Box::new(QueueNode {
            entry: QueueEntry::Process { dir, entries },
            next,
        }));
    }

    fn pump(&mut self, chan: &SyncSender<io::Result<WalkEntry>>) -> io::Result<i32> {
        match self.next_task() {
            QueueTask::Scan(dir) => {
                let entries = self.scan_dir(&dir)?;
                self.push_dir_queue(dir, entries);
                Ok(0)
            }

            QueueTask::Process { dir, entry } => {
                let w = self.scan_entry(dir, entry)?;
                chan.send(Ok(w)).expect("receiver hung up");
                Ok(1)
            }

            QueueTask::Noop => Ok(0),

            QueueTask::Finished => Ok(-1),
        }
    }

    fn next_task(&mut self) -> QueueTask {
        if let Some(mut node) = self.stack.take() {
            match node.entry {
                QueueEntry::Scan(dir) => {
                    self.stack = node.next;
                    QueueTask::Scan(dir)
                }
                QueueEntry::Process {
                    ref dir,
                    ref mut entries,
                } => {
                    if let Some(entry) = entries.pop_front() {
                        let dir = dir.clone();
                        self.stack = Some(node);
                        QueueTask::Process { dir, entry }
                    } else {
                        self.stack = node.next;
                        QueueTask::Noop
                    }
                }
            }
        } else {
            QueueTask::Finished
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
            entries.push(ent);
        }
        entries.sort_by_key(|e| e.file_name());
        Ok(entries.into())
    }

    fn scan_entry(&mut self, dir: PathBuf, entry: DirEntry) -> io::Result<WalkEntry> {
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

        if file_type.is_dir() {
            self.push_dir(path.clone());
        }

        let w = WalkEntry {
            path,
            file_type,
            size: meta.len(),
            mtime: meta.modified().ok(),
            ctime: meta.created().ok(),
        };
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
