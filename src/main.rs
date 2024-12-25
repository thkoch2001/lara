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
use async_shutdown::ShutdownManager;
use reqwest::Url;
use std::process::exit;

const BOTNAME: &str = "larabot";

/// ShutdownManager is needed so that GzipEncoder.shutdown() can be called on
/// error or interrupt. There is no good story yet in Rust for async drop:
/// <https://trouble.mataroa.blog/blog/asyncawait-is-real-and-can-hurt-you/>
/// TODO: I've no idea whether I use async_shutdown correctly!
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("starting up");

    let shutdown = ShutdownManager::new();

    // handle interrupt signal
    tokio::spawn({
        let shutdown = shutdown.clone();
        async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                info!("Failed to wait for interrupt signal: {}", e);
                exit(1);
            } else {
                info!("Received interrupt signal. Trigger shutdown.");
                shutdown.trigger_shutdown(0).ok();
            }
        }
    });

    let mut crawler = crawler::Crawler::new(BOTNAME);
    crawler
        .run(
            shutdown.clone(),
            Url::parse("https://de.populus.wiki").unwrap(),
        )
        .await;
    Ok(())
}
