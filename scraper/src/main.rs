use reqwest::blocking::Client;
use reqwest::Error;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let params = vec![
        ("list".to_string(), "categorymembers".to_string()),
        ("cmtitle".to_string(), "Category:Is_a_snp".to_string())];
    let request = MWRequest::query_json(params);
    let pager: MWSource<SNPListPage> = MWSource::new(&client, request);
    let results: Vec<Result<Vec<SNPData>, Error>> = pager.into_iter().take(3).map(|page| handle_page(&client, page)).collect();

    println!("{:?}", results);
    Ok(())
}

fn handle_page(client: &Client, page: Result<SNPListPage, Error>) -> Result<Vec<SNPData>,Error> {
    let snps = page?.categorymembers.into_iter().map(|m| m.title).collect();
    let req = MWRequest::get_titles(snps);
    let rsp: MWResponse<SNPBatchPage> = req.send_request(&client)?;
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
        SNPData { name, content: json["revisions"][0]["slots"]["main"]["*"].as_str().unwrap().to_string()}
    }
}

#[derive(Deserialize, Debug, Clone)]
struct MWContinue { cmcontinue: String, r#continue: String }

impl MWContinue {
    fn to_params(&self) -> Vec<(&str, &str)> {
        vec!(("cmcontinue", self.cmcontinue.as_str()), ("continue", self.r#continue.as_str()))
    }
}

#[derive(Deserialize, Debug)]
struct MWResponse<T> {
    r#continue: Option<MWContinue>,
    batchcomplete: String,
    query: T, 
}

type Params = Vec<(String, String)>;

struct MWRequest {
    cont: Option<MWContinue>,
    params: Params
}

impl MWRequest {
    fn new(params: Params) -> MWRequest {
        MWRequest { cont: None, params }
    }

    fn query_json(mut params: Params) -> MWRequest {
        params.push(("action".to_string(), "query".to_string()));
        params.push(("format".to_string(), "json".to_string()));
        MWRequest::new(params)
    }

    fn get_titles(titles: Vec<String>) -> MWRequest {
        let params = vec![
            ("action".to_string(), "query".to_string()),
            ("format".to_string(), "json".to_string()),
            ("prop".to_string(), "revisions".to_string()),
            ("rvprop".to_string(), "content|timestamp".to_string()),
            ("rvslots".to_string(), "main".to_string()),
            ("titles".to_string(), titles.join("|"))
        ];

        MWRequest::new(params)
    }

    fn send_request<T: DeserializeOwned>(&self, client: &Client) -> Result<MWResponse<T>, Error> {
        let mut builder = client
            .get("http://bots.snpedia.com/api.php")
            .query(&self.params);

        builder = match &self.cont {
            None => builder,
            Some(cont) => builder.query(&cont.to_params())
        };

        builder.send()?.json()        
    }    
}

struct MWSource<'a, T> {
    client: &'a Client,
    request: Option<MWRequest>,
    _marker: std::marker::PhantomData<T>
}

impl<'a, T> MWSource<'a, T> {
    fn new(client: &'a Client, request: MWRequest) -> MWSource<'a, T> {
        MWSource { client, request: Some(request), _marker: std::marker::PhantomData }
    }
}

impl<'a, T> Iterator for MWSource<'a, T> where T: DeserializeOwned + std::fmt::Debug {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.request {
            None => None,
            Some(req) => {
                let rsp: Result<MWResponse<T>, Error> = req.send_request(&self.client);
                match &rsp {
                    Ok(mwrsp) => {
                        match &mwrsp.r#continue {
                            Some(cont) => req.cont = Some(cont.clone()),
                            None => self.request = None
                        }
                    },
                    Err(_) => self.request = None
                };                
                Some(rsp.map(|r| r.query))
            }
        }
    }
}




