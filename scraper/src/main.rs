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
    let req = MWRequest::new(params);
    let rsp: MWResponse<HashMap<String, serde_json::Value>> = send_request(&client, req)?;
    println!("{:?}", rsp);
    Ok(())
}

#[derive(Deserialize, Debug)]
struct MWContinue { cmcontinue: String, r#continue: String }

impl MWContinue {
    fn to_params(cont: MWContinue) -> Params {
        vec!(("cmcontinue".to_string(), cont.cmcontinue), ("continue".to_string(), cont.r#continue))
    }
}

#[derive(Deserialize, Debug)]
struct MWResponse<T> {
    r#continue: MWContinue,
    batchcomplete: String,
    query: T, 
}

impl<T> MWResponse<T> {
    fn is_complete(&self) -> bool {
        self.batchcomplete == "done"
    }
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
}


fn send_request<T>(client: &Client, request: MWRequest) -> Result<MWResponse<T>, Error>
    where T: DeserializeOwned {
    let common_params = [
        ("action", "query"),
        ("format", "json")];

    let mut builder = client
        .get("http://bots.snpedia.com/api.php")
        .query(&common_params)
        .query(&request.params);

    builder = match request.cont {
        None => builder,
        Some(cont) => builder.query(&MWContinue::to_params(cont))
    };

    builder
        .send()?
        .json()
}
