use rayon::prelude::*;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

static BYTE_FIXES: &[(&[u8], &'static str)] = &[
    (b"&nbsp;", ""),
    (b"&quot;", "\""),
    (b"&amp;", "&"),
    (b"\xF8", "ø"),
    (b"&#248;", "ø"),
    (b"\xC3\xB8", "ø"),
    (b"\xD8", "Ø"),
    (b"&#216;", "Ø"),
    (b"\xE6", "æ"),
    (b"\xC3\xA6", "æ"),
    (b"&#230;", "æ"),
    (b"\xC6", "Æ"),
    (b"&#198;", "Æ"),
    (b"\xE5", "å"),
    (b"\xC5", "Å"),
    (b"\xEF\xBF\xBD", "#{INVALID_CHAR}#"),
    (b"&#229;", "å"),
    (b"&#197;", "Å"),
    (b"\xB4", "'"),
    (b"\xBD", "½"),
];

fn replace<T>(source: &[T], from: &[T], to: &[T]) -> Vec<T>
where
    T: Clone + PartialEq,
{
    let mut result = source.to_vec();
    let from_len = from.len();
    let to_len = to.len();

    let mut i = 0;
    while i + from_len <= result.len() {
        if result[i..].starts_with(from) {
            result.splice(i..i + from_len, to.iter().cloned());
            i += to_len;
        } else {
            i += 1;
        }
    }

    result
}

fn recurse(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else { return vec![] };
    entries
        .flatten()
        .flat_map(|entry| {
            let Ok(meta) = entry.metadata() else { return vec![] };
            if meta.is_dir() {
                return recurse(entry.path());
            }
            if meta.is_file() {
                return vec![entry.path()];
            }
            vec![]
        })
        .collect()
}

fn process_file(path: &str) {
    if path.ends_with(".jpg") || path.ends_with(".png") || path.ends_with(".gif") {
        return;
    }
    let mut file = fs::read(path).unwrap();
    BYTE_FIXES.iter().for_each(|(bad, good)| {
        file = replace(&file, bad, good.as_bytes());
    });
    fs::write(path, file).unwrap();
}

fn multi_threaded() -> io::Result<()> {
    let paths: Vec<_> = recurse("../volvo240.dk");
    paths
        .par_iter()
        .for_each(|entry| process_file(entry.to_str().unwrap()));
    Ok(())
}

fn main() -> io::Result<()> {
    multi_threaded()
}
