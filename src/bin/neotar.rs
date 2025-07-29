use std::{env, ffi::CStr, fs, path::PathBuf};

fn write_root_dir(path: PathBuf, bytes: &mut Vec<u8>) {
    let entries = fs::read_dir(&path).unwrap();
    let len_entries = fs::read_dir(&path).unwrap();
    let len = len_entries
        .filter(|x| !x.as_ref().unwrap().file_type().unwrap().is_symlink())
        .count();
    bytes.extend_from_slice(&(len as u32).to_be_bytes());
    for entry in entries {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        if ty.is_symlink() {
            continue;
        } else if ty.is_dir() {
            write_dir(entry.path(), bytes);
        } else if ty.is_file() {
            let mut name = entry.file_name().into_encoded_bytes();
            name.push(0);
            neotar::write_single_entry(
                &neotar::Entry {
                    name: CStr::from_bytes_with_nul(&name).unwrap(),
                    entry: neotar::EntryInner::File(&fs::read(entry.path()).unwrap()),
                },
                bytes,
            );
        }
    }
}

fn write_dir(path: PathBuf, bytes: &mut Vec<u8>) {
    let mut name = path.file_name().unwrap().as_encoded_bytes().to_owned();
    name.push(0);
    let entries = fs::read_dir(&path).unwrap();
    let len_entries = fs::read_dir(&path).unwrap();
    let len = len_entries
        .filter(|x| !x.as_ref().unwrap().file_type().unwrap().is_symlink())
        .count();
    neotar::write_single_entry(
        &neotar::Entry {
            name: CStr::from_bytes_with_nul(&name).unwrap(),
            entry: neotar::EntryInner::Directory(len as u32),
        },
        bytes,
    );
    for entry in entries {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        if ty.is_symlink() {
            continue;
        } else if ty.is_dir() {
            write_dir(entry.path(), bytes);
        } else if ty.is_file() {
            let mut name = entry.file_name().into_encoded_bytes();
            name.push(0);
            neotar::write_single_entry(
                &neotar::Entry {
                    name: CStr::from_bytes_with_nul(&name).unwrap(),
                    entry: neotar::EntryInner::File(&fs::read(entry.path()).unwrap()),
                },
                bytes,
            );
        }
    }
}

fn main() {
    let path = env::current_dir().unwrap();
    let mut bytes = vec![];
    write_root_dir(path, &mut bytes);
    fs::write("a.ntar", bytes).unwrap();
}
