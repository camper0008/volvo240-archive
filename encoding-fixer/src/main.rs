use rayon::prelude::*;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};

fn recurse(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return vec![];
    };
    entries
        .flatten()
        .flat_map(|entry| {
            let Ok(meta) = entry.metadata() else {
                return vec![];
            };
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
    let file = fs::read(path).unwrap();
    let file = ISO_8859_1.decode(&file, DecoderTrap::Strict).unwrap();
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
