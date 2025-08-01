use std::ffi::CStr;

use crate::Section;

#[derive(Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    pub name: &'a CStr,
    pub entry: EntryInner<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryInner<'a> {
    File(&'a [u8]),
    Directory(u32),
}

fn read_single_entry(bytes: &[u8]) -> (Entry, usize) {
    let mut i = 0;
    let is_dir = bytes[i] != 0;
    i += 1;
    while bytes[i] != 0 {
        i += 1;
    }
    let name = CStr::from_bytes_with_nul(&bytes[1..=i]).unwrap();
    i += 1;
    let len = u32::from_be_bytes(bytes[i..(i + 4)].try_into().unwrap());
    i += 4;
    (
        Entry {
            name,
            entry: if !is_dir {
                let entry = EntryInner::File(&bytes[i..i + (len as usize)]);
                i += len as usize;
                entry
            } else {
                EntryInner::Directory(len)
            },
        },
        i,
    )
}
pub fn read_entry_recursive<'a>(bytes: &'a [u8], entries: &mut Vec<Entry<'a>>) -> usize {
    let mut i = 0;
    let mut remaining = 1;
    while remaining > 0 {
        let (entry, j) = read_single_entry(&bytes[i..]);
        i += j;
        if let EntryInner::Directory(len) = &entry.entry {
            remaining += len
        }
        entries.push(entry);
        remaining -= 1;
        dbg!(remaining);
    }
    dbg!(i);
    i
}

pub fn write_single_entry(entry: &Entry, bytes: &mut Vec<u8>) {
    let name = entry.name;
    let entry = &entry.entry;
    match entry {
        EntryInner::File(contents) => {
            bytes.push(0);
            bytes.extend_from_slice(name.to_bytes_with_nul());
            bytes.extend_from_slice(&(contents.len() as u32).to_be_bytes());
            bytes.extend_from_slice(contents);
        }
        EntryInner::Directory(len) => {
            bytes.push(1);
            bytes.extend_from_slice(name.to_bytes_with_nul());
            bytes.extend_from_slice(&len.to_be_bytes());
        }
    }
}
pub fn write_entry_recursive(entries: &[Entry], bytes: &mut Vec<u8>) {
    for entry in entries {
        write_single_entry(entry, bytes);
    }
}
pub fn read_entries_recursive<'a>(section: &Section<'a>) -> (Vec<Entry<'a>>, u32) {
    let mut entries = vec![];
    let mut i = 4;
    while i < section.bytes.len() {
        i += read_entry_recursive(&section.bytes[i..], &mut entries);
    }
    (entries, u32::from_be_bytes(section.bytes[..4].try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use std::{ffi::CString, str::FromStr};

    use super::*;

    #[test]
    fn file_read_single_entry() {
        let bytes = [0, b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 1, 65];
        let entry = read_single_entry(&bytes).0;
        assert_eq!(
            entry,
            Entry {
                name: &CString::from_str("hello").unwrap(),
                entry: EntryInner::File(&[65])
            }
        )
    }
    #[test]
    fn rw_entry() {
        let entries = [
            Entry {
                name: &CString::from_str("hello").unwrap(),
                entry: EntryInner::Directory(1),
            },
            Entry {
                name: &CString::from_str("world.txt").unwrap(),
                entry: EntryInner::File(&[0, 1, 2, 3, 4]),
            },
        ];
        let mut bytes = vec![];
        write_entry_recursive(&entries, &mut bytes);
        let mut new = vec![];
        read_entry_recursive(&bytes, &mut new);
        assert_eq!(entries.as_slice(), &new)
    }
}
