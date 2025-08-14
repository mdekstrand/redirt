//! File system walks with [ignore].
use std::path::{Path, PathBuf};

use ignore::{Walk, WalkBuilder};

use crate::walk::{TreeWalk, WalkEntry, WalkError, WalkOptions};

/// File system walker.
pub struct FSWalk {
    root: PathBuf,
    walk: Walk,
}

pub fn walk_fs<P: AsRef<Path>>(root: P, options: &WalkOptions) -> FSWalk {
    let root = root.as_ref().to_path_buf();
    let mut wb = WalkBuilder::new(&root);
    wb.ignore(!options.no_ignore);
    wb.follow_links(options.follow_symlinks);
    wb.hidden(options.include_hidden);
    wb.sort_by_file_name(|f1, f2| f1.cmp(f2));
    let walk = wb.build();
    FSWalk { root, walk }
}

impl Iterator for FSWalk {
    type Item = Result<WalkEntry, WalkError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.walk.next().map(|e| {
            let entry = e?;
            Ok(WalkEntry {
                path: entry.path().strip_prefix(self.root())?.to_owned(),
                meta: Some(entry.metadata()?),
            })
        })
    }
}

impl TreeWalk for FSWalk {
    fn root(&self) -> &Path {
        &self.root
    }
}
