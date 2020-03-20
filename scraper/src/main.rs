use reqwest::blocking::Client;
use reqwest::Error;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let params = [
        ("list", "categorymembers"),
        ("cmtitle", "Category:Is_a_snp")];
    let req = MWRequest::create(&params);
    let rsp: MWResponse<HashMap<String, serde_json::Value>> = send_request(&client, &req)?;
    println!("{:?}", rsp);
    Ok(())
}

#[derive(Deserialize, Debug)]
struct MWContinue { cmcontinue: String, r#continue: String }

impl MWContinue {
    fn params(&self) -> [(&str, &str); 2] {
        [("cmcontinue", self.cmcontinue.as_str()), ("continue", self.r#continue.as_str())]
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

struct MWRequest<'a> {
    cont: Option<MWContinue>,
    params: &'a[(&'a str, &'a str)]
}

impl MWRequest<'_> {
    fn create<'a>(params: &'a[(&'a str, &'a str)]) -> MWRequest<'a> {
        MWRequest { cont: None, params }
    }
}


fn send_request<T>(client: &Client, request: &MWRequest) -> Result<MWResponse<T>, Error>
    where T: DeserializeOwned {
    let common_params = [
        ("action", "query"),
        ("format", "json")];

    let mut builder = client
        .get("http://bots.snpedia.com/api.php")
        .query(&common_params)
        .query(request.params);

    builder = match &request.cont {
        None => builder,
        Some(cont) => builder.query(&cont.params())
    };

    builder
        .send()?
        .json()
}
