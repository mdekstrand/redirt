use std::{fs::DirEntry, io};

pub fn is_hidden(de: &DirEntry) -> io::Result<bool> {
    let name = de.file_name();
    let s = name.to_string_lossy();
    Ok(s.starts_with("."))
}
