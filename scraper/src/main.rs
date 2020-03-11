use reqwest::{Client, Response, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let rsp = send_request(&client).await?;
    println!("{:?}", rsp);
    Ok(())
}

async fn send_request(client: &Client) -> Result<Response, Error> {
    let params = [
        ("list", "categorymembers"),
        ("cmtitle", "Category:Is_a_snp")];
    client
        .get("http://bots.snpedia.com/api.php")
        .query(&params)
        .send()
        .await
}
