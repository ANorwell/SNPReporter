use std::collections::HashMap;

use std::io::{BufRead, BufReader, Error, Read};
use std::hash::Hash;

#[derive(Hash,PartialEq, Eq, Debug)]
pub struct Snp {
  id: String
}

#[derive(Hash,PartialEq, Eq, Debug)]
pub struct Genotype {
  name: String
}

pub fn parse<R: Read>(input: BufReader<R>) -> Result<HashMap<Snp, Genotype>, Error> {
  let mut map = HashMap::new();
  for line in input.lines().take(50) {
    let (snp, genotype) = line.and_then(parse_line)?;
    map.insert(snp, genotype);
  }

  Ok(map)
}

fn parse_line(line: String) -> Result<(Snp, Genotype), Error> {
  Ok((Snp { id: line }, Genotype { name: String::from("GG") }))
}
