use std::os::windows::fs::MetadataExt;
use std::{fs::DirEntry, io};
use winapi::um::winnt::FILE_ATTRIBUTE_READONLY;

pub fn is_hidden(de: &DirEntry) -> io::Result<bool> {
    let meta = de.metadata()?;
    let attrs = meta.file_attributes();
    Ok(attrs & FILE_ATTRIBUTE_READONLY != 0)
}
