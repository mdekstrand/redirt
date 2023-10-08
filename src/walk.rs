//! Walk file trees.

use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread::JoinHandle;
use std::{io, thread};

const DEFAULT_BUFFER: usize = 1000;

/// Set up a tree-walker.
pub struct WalkBuilder {
    root: PathBuf,
    buf_size: usize,
}

/// Implementation of tree-walking.
pub struct Walker {
    channel: Receiver<WalkEntry>,
    handle: JoinHandle<io::Result<usize>>,
}

/// Single result entry in a tree-walk.
pub struct WalkEntry {}

impl WalkBuilder {
    /// Create a walk builder for a root directory.
    pub fn for_directory<P: AsRef<Path>>(root: P) -> WalkBuilder {
        WalkBuilder {
            root: root.as_ref().to_owned(),
            buf_size: DEFAULT_BUFFER,
        }
    }

    /// Start walking the directory.
    pub fn walk(self) -> Walker {
        let (send, recv) = sync_channel(self.buf_size);
        let handle = thread::spawn(move || self.walk_worker(send));
        Walker {
            channel: recv,
            handle,
        }
    }

    fn walk_worker(self, send: SyncSender<WalkEntry>) -> io::Result<usize> {
        Ok(0)
    }
}

impl Iterator for Walker {
    type Item = WalkEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl WalkEntry {}
