#![warn(clippy::all, clippy::pedantic)]

mod url_frontier;

use reqwest::{Error, Url};
use url_frontier::{UrlFrontier, UrlFrontierVec};

async fn get_request(mut url_frontier: impl UrlFrontier) -> Result<(), Error> {
    while let Some(url) = url_frontier.get_url() {
        let response = reqwest::get(url).await?;
        println!("Status: {}", response.status());

        let body = response.text().await?;
        println!("Body:\n{body}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut url_frontier = UrlFrontierVec::new();
    url_frontier.put_url(Url::parse("https://populus.wiki").unwrap());
    get_request(url_frontier).await?;
    Ok(())
}
