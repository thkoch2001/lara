#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate log;

mod fetcher;
mod robots_txt;
mod url_frontier;

use anyhow::Result;
use fetcher::Fetcher;
use reqwest::{/*Error, */ Url};
use robots_txt::CheckResult;
use url_frontier::{UrlFrontier, UrlFrontierVec};

const BOTNAME: &str = "larabot";

async fn crawl(mut url_frontier: impl UrlFrontier) -> Result<()> {
    let fetcher = Fetcher::new(String::from(BOTNAME));
    let mut robots_txt_manager = robots_txt::Manager::new(BOTNAME.as_ref());

    while let Some(url) = url_frontier.get_url() {
        match robots_txt_manager.check(&url, &fetcher).await? {
            CheckResult::Allowed => (),
            CheckResult::Disallowed => {
                info!("Crawling of {url} forbidden by robots.txt");
                continue;
            }
            CheckResult::Retry(seconds) => {
                info!("Retry robots check for {url} in {seconds}s");
                todo!("put back url for retry in seconds");
                //continue;
            }
        }

        info!("Crawling {url}");
        let response = fetcher.fetch(url.clone()).await?;
        info!("Status: {} for {url}", response.status());

        let body = response.text().await?;
        println!("Body:\n{body}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("starting up");
    let mut url_frontier = UrlFrontierVec::new();
    url_frontier.put_url(Url::parse("https://populus.wiki").unwrap());
    crawl(url_frontier).await?;
    Ok(())
}
