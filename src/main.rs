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
mod fetcher;
mod robotstxt_cache;
mod sitemaps;
mod url_frontier;

use env_config::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use url::Url;

env_vars![
    BOT_NAME
    ARCHIVE_DIR
];

fn ctrlc_flag() -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    let flag_for_handler = Arc::clone(&flag);

    ctrlc::set_handler(move || {
        flag_for_handler.store(true, Ordering::Relaxed);
        info!("interrupting...");
    })
    .expect("Failed to set ctrlc handler");
    flag
}

fn main() {
    env_logger::init();
    info!("starting up");

    env_config::check();
    info!("Configuration: {:?}", env_config::get_map());

    let t = thread::spawn(move || {
        let mut crawler = crawler::Crawler::new(ctrlc_flag());
        if let Err(e) = crawler.run(&Url::parse("https://de.populus.wiki").unwrap()) {
            error!("{e:?}");
        }
    });

    info!("waiting for crawler");
    let _ = t.join();
}
