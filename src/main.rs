//! Crawler
//!
//! - Consumes urls from `url_frontier`
//! - respects robots.txt
//! - checks sitemaps.xml
//! - writes WARC archive of crawled urls

#![warn(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]

// TODO https://doc.rust-lang.org/rustdoc/lints.html
// TODO study https://support.google.com/webmasters/answer/7440203 Page Indexing report

#[macro_use]
extern crate log;

mod clock;
mod crawler;
#[macro_use]
mod env_vars;
mod db;
mod fetcher;
mod link_extractor;
mod robotstxt;
mod signal_handler;
mod url_frontier;
mod url_util;

use std::thread;

env_vars![
    ARCHIVE_DIR
    BOT_NAME
    DB_URL
    FROM // https://httpwg.org/specs/rfc9110.html#field.from
];

fn main() {
    env_logger::init();
    info!("starting up");

    env_config::check();
    info!("Configuration: {:?}", env_config::get_map());

    let t = thread::spawn(move || {
        let signal_handler = signal_handler::SignalHandler::register();

        let mut crawler = crawler::Crawler::new(signal_handler).expect("error");
        if let Err(e) = crawler.run() {
            error!("{e:?}");
        }
    });

    info!("waiting for crawler");
    let _ = t.join();
}
