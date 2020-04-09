use reqwest::blocking::Client;
use serde::Deserialize;

use std::collections::HashMap;
use std::fmt;

use std::fs::File;
use std::io::prelude::*;
use std::io::{Result as IOResult};

mod mediawiki;

use mediawiki::{ MWRequest, MWResponse, MWSource };

const DATA_DIR: &str = "data";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir(DATA_DIR)?;
    
    let client = Client::new();
    let params = vec![
        ("list".to_string(), "categorymembers".to_string()),
        ("cmtitle".to_string(), "Category:Is_a_snp".to_string())];
    let request = MWRequest::query_json(params);
    let pager: MWSource<SNPListPage> = MWSource::new(&client, request);
    let results: Vec<Result<(), SNPError>> =
     pager
        .into_iter()
        .take(3)
        .flat_map(|page| page.map_err(|e| SNPError::Net(e)).map(|p| handle_page(&client, p)))
        .collect();

    println!("{:#?}", results);
    Ok(())
}

fn handle_page(client: &Client, page: SNPListPage) -> Result<(),SNPError> {
    let snps = page.categorymembers.into_iter().map(|m| m.title).collect();
    let req = MWRequest::get_titles(snps);
    let rsp: MWResponse<SNPBatchPageSet> = req.send(&client).map_err(|e| SNPError::Net(e))?;
    
    rsp.query
        .to_snp_data()?
        .into_iter()
        .map(|data| store(data))
        .collect()

}

fn store(data: SNPData) -> Result<(), SNPError> {
    let name = data.name;
    let write_result = write_snp(&name, data.content);
    write_result.map_err(|e|  SNPError::WriteError { name, error: e } )
}

fn write_snp(name: &String, content: String) -> IOResult<()> {
    let mut file = File::create(format!("{}/{}", DATA_DIR, name))?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct SNPListMember { title: String }
#[derive(Deserialize, Debug)]
struct SNPListPage { categorymembers: Vec<SNPListMember> }

#[derive(Deserialize, Debug)]
struct SNPBatchPageSet { pages: HashMap<String, serde_json::Value> }
impl SNPBatchPageSet {
    fn to_snp_data(self) -> Result<Vec<SNPData>, SNPError> {
        self.pages.into_iter().map(|(k, v)| SNPData::parse(k, v)).collect()
    }
}

#[derive(Debug)]
enum SNPError {
    ParseError(),
    WriteError { name: String, error: std::io::Error },
    Net(reqwest::Error)
}

impl std::error::Error for SNPError {}

impl fmt::Display for SNPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }    
}


#[derive(Debug)]
struct SNPData { name: String, content: String }
impl SNPData {
    fn parse(name: String, json: serde_json::Value) -> Result<SNPData, SNPError> {
        json["revisions"][0]["slots"]["main"]["*"].as_str().ok_or(SNPError::ParseError())
            .map(|c| SNPData { name, content: c.to_string() })
    }
}

