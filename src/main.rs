#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate log;

mod robots_txt;
mod url_frontier;

use anyhow::Result;
use reqwest::{/*Error, */ Url};
use robots_txt::CheckResult;
use url_frontier::{UrlFrontier, UrlFrontierVec};

const BOTNAME: &str = "larabot";

async fn get_request(mut url_frontier: impl UrlFrontier) -> Result<()> {
    let robots_txt_manager = robots_txt::Manager::new(BOTNAME);
    while let Some(url) = url_frontier.get_url() {
        let robots_check = robots_txt_manager.check(&url).await?;
        if robots_check != CheckResult::Allowed {
            info!("Crawling of {url} forbidden by robots.txt");
            continue;
        }
        info!("Crawling {url}");
        let response = reqwest::get(url.clone()).await?;
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
    get_request(url_frontier).await?;
    Ok(())
}
