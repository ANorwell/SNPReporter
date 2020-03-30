use reqwest::blocking::Client;
use reqwest::Error;
use serde::Deserialize;

use std::collections::HashMap;

mod mediawiki;

use mediawiki::{ MWRequest, MWResponse, MWSource };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let params = vec![
        ("list".to_string(), "categorymembers".to_string()),
        ("cmtitle".to_string(), "Category:Is_a_snp".to_string())];
    let request = MWRequest::query_json(params);
    let pager: MWSource<SNPListPage> = MWSource::new(&client, request);
    let results: Vec<Result<Vec<SNPData>, Error>> = pager.into_iter().take(3).map(|page| handle_page(&client, page)).collect();

    println!("{:#?}", results);
    Ok(())
}

fn handle_page(client: &Client, page: Result<SNPListPage, Error>) -> Result<Vec<SNPData>,Error> {
    let snps = page?.categorymembers.into_iter().map(|m| m.title).collect();
    let req = MWRequest::get_titles(snps);
    let rsp: MWResponse<SNPBatchPage> = req.send(&client)?;
    Ok(rsp.query.pages.into_iter().map(|(k, v)| SNPData::parse(k, v) ).collect())
}

#[derive(Deserialize, Debug)]
struct SNPListMember { title: String }
#[derive(Deserialize, Debug)]
struct SNPListPage { categorymembers: Vec<SNPListMember> }

#[derive(Deserialize, Debug)]
struct SNPBatchPage { pages: HashMap<String, serde_json::Value> }

#[derive(Debug)]
struct SNPData { name: String, content: String }
impl SNPData {
    fn parse(name: String, json: serde_json::Value) -> SNPData {
        SNPData { name, content: json["revisions"][0]["slots"]["main"]["*"].as_str().unwrap().to_string() }
    }
}

