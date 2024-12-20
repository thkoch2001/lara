#![warn(clippy::all, clippy::pedantic)]

// TODO study https://support.google.com/webmasters/answer/7440203 Page Indexing report

#[macro_use]
extern crate log;

mod clock;
mod crawler;
mod fetcher;
mod robotstxt_cache;
mod sitemaps;
mod url_frontier;

use anyhow::Result;
use reqwest::Url;

const BOTNAME: &str = "larabot";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("starting up");

    let mut crawler = crawler::Crawler::new(BOTNAME);
    crawler
        .run(Url::parse("https://de.populus.wiki").unwrap())
        .await?;
    Ok(())
}
