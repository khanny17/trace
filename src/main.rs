extern crate grep;
extern crate walkdir;
extern crate memmap;
extern crate regex;

use regex::Regex;

use std::fs::File;
use walkdir::WalkDir;
use grep::Grep;
use grep::GrepBuilder;
use grep::Match;

use memmap::Mmap;

fn should_ignore(path: &str) -> bool {
    let re = Regex::new(r"^(\./|/)target|/\..*").expect("Unable to parse regex");
    return re.find(path).is_some();
}

fn search_file(file: &File, search: &Grep) -> bool {
    let mmap = unsafe { Mmap::map(&file) };
    if mmap.is_err() {
        eprintln!("failed to map the file: {}", mmap.unwrap_err());
        return false;
    }

    let mmap = mmap.unwrap();

    let mut found_at_least_one = false;
    let mut the_match = Match::new();
    let mut start_index = 0;
    loop {
        let found = search.read_match(&mut the_match, &mmap[..], start_index);
        if found {
            found_at_least_one = true;
            println!("{:?}", the_match);
            start_index = the_match.end();
        } else {
            break;
        }
    }

    return found_at_least_one;
}

fn walk_dir(dir: &str, search: &Grep) {
    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();

        // Don't run on non-files
        if !entry.file_type().is_file() {
            continue;
        }

        // Skip ignore paths
        if should_ignore(entry.path().to_str().unwrap()) {
            continue;
        }

        let file = File::open(entry.path());

        // Skip if we failed to open the file
        if file.is_err() {
            eprintln!("Unable to open {}: {}",
                      entry.path().display(),
                      file.unwrap_err());
            continue;
        }

        let file = file.unwrap();
        if search_file(&file, &search) {
            println!("{}", entry.path().display());
        }
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or(String::from("."));

    let search = GrepBuilder::new("mongoose")
        .build()
        .expect("Unable to build search");
    walk_dir(&path, &search);
}
