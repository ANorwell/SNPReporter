mod parser;

use std::io::{BufReader, Error};
use std::fs::File;

fn main() {
    println!("Hello, world!");
    run().expect("error running parsing");
}

fn run() -> Result<(), Error> {
    let f = File::open("data/23andme.txt")?;
    let input = BufReader::new(f);
    let manifest = parser::parse(input)?;
    println!("{:?}", manifest);
    Ok(())
}

