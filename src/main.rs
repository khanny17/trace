extern crate grep;

use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    println!("Hello, world!");

    let builder = grep::GrepBuilder::new("Hello");

    let built = builder.build();
    let mut the_match = grep::Match::new();

    let mut file = File::open("./src/main.rs").unwrap();
    let mut buffer = [0; 500];

    file.read(&mut buffer[..]);

    let found = built.unwrap().read_match(&mut the_match, &buffer, 0);

    println!("{:?}", found);
    println!("{:?}", the_match);
}
