#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

extern crate grep;
extern crate walkdir;
extern crate memmap;
extern crate regex;

extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use regex::Regex;

use std::fs::File;
use walkdir::WalkDir;
use grep::Grep;
use grep::GrepBuilder;
use grep::Match;
use std::ops::Deref;
use std::io::BufRead;

use memmap::Mmap;

fn should_ignore(path: &str) -> bool {
    // This lets us only instantiate the regex one time, which is expensive
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\./|/)target|/\..*")
            .expect("Unable to parse regex");
    }

    return RE.find(path).is_some();
}

fn search_file(file: &File, search: &Grep) -> Result<Vec<(u32, String)>, String> {
    let mmap = unsafe { Mmap::map(&file) };
    if mmap.is_err() {
        return Err(format!("failed to map the file: {}", mmap.unwrap_err()));
    }

    let mut findings = Vec::new();

    let mut line_number = 0;
    let mmap = mmap.unwrap();
    for line in mmap.deref().lines() {
        if line.is_err() {
            continue;
        }

        let line = line.unwrap();
        let mut the_match = Match::new();
        let found = search.read_match(&mut the_match, line.as_bytes(), 0);
        if found {
            findings.push( (line_number, line) );
        }
        line_number += 1;
    }

    return Ok(findings);
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
        let findings = search_file(&file, search).unwrap_or(Vec::new());
        if !findings.is_empty() {
            println!("{}", entry.path().display());
            println!("{:?}", findings);
        }
    }
}

// Read in requirements yaml files, linting for correctness
#[derive(Serialize, Deserialize, Debug)]
struct Requirement {
    id: String, // The prefix before the trace in source files
    name: String,
    details: Option<String>,
    parent: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct RequirementCollection {
    requirements: Vec<Requirement>,
}
impl RequirementCollection {
    fn new() -> RequirementCollection {
        RequirementCollection { requirements: Vec::new() }
    }
}

fn parse_requirements(dir: String) -> RequirementCollection {
    let mut collection = RequirementCollection::new();
    for entry in WalkDir::new(dir) {
        if entry.is_err() {
            eprintln!("{}", entry.unwrap_err());
            continue;
        }

        let entry = entry.unwrap();

        // Don't run on non-files
        if !entry.file_type().is_file() {
            continue;
        }

        // Skip ignore paths
        let extension = entry.path().extension();
        if extension.is_none() || extension.unwrap() != "yml" {
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
        let requirement: Result<Requirement, serde_yaml::Error> =
            serde_yaml::from_reader(file);

        if requirement.is_err() {
            eprintln!("Unable to parse {}: {}",
                      entry.path().display(),
                      requirement.unwrap_err());
            continue;
        }

        let requirement = requirement.unwrap();
        collection.requirements.push(requirement);
    }
    return collection;
}

#[derive(Serialize, Deserialize, Debug)]
struct TraceConfig {
    identifier: String, // The prefix before the trace in source files
    requirements_dir: String,
}
impl TraceConfig {
    fn new() -> TraceConfig {
        TraceConfig {
            identifier: String::from("~tr:"),
            requirements_dir: String::from("req"),
        }
    }
}
fn read_config() -> TraceConfig {
    let file = File::open(".trace.yml");
    if file.is_err() {
        eprintln!("No .trace.yml file found");
        return TraceConfig::new();
    }

    return serde_yaml::from_reader(file.unwrap()).unwrap_or_else(|e| {
        eprintln!("Unable to parse .trace.yml: {}", e);
        return TraceConfig::new();
    });
}

fn main() {
    let config = read_config();
    println!("{:?}", config);

    let path = std::env::args()
        .nth(1)
        .unwrap_or(String::from("."));

    let requirements = parse_requirements(format!("{}/{}", path, config.requirements_dir));
    println!("{:?}", requirements);

    let ref_search = GrepBuilder::new(config.identifier.as_str())
        .build()
        .expect("Unable to build reference search");
    walk_dir(&path, &ref_search);
}
