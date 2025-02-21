//! URL frontier tells crawlers which URL to crawl next and eliminates duplicate
//! URLs.
//!
//! See also:
//! - [URL Frontier chapter][itir] in Introduction to Information Retrieval
//! - [Crawler-Commons URL frontier](https://github.com/crawler-commons/url-frontier)
//! - [Frontera](https://frontera.readthedocs.io)
//! - [Wikipedia: Crawl frontier](https://en.wikipedia.org/wiki/Crawl_frontier)
//!
//! [itir]: https://nlp.stanford.edu/IR-book/html/htmledition/the-url-frontier-1.html

use crate::db;
use crawl::fetch::FetchResult;
use anyhow::Result;
use diesel::pg::PgConnection;
use url::Url;

use crate::crawler::{Outlink, UrlItem};

pub struct UrlFrontier {
    conn: PgConnection,
    current_url_id: i32,
    domain: String,
    urls: Vec<(i32, Url)>,
    /// urls received already from the DB to be excluded from SELECTs
    url_ids_received: Vec<i32>,
}

impl UrlFrontier {
    pub fn new() -> Result<Self> {
        Ok(UrlFrontier {
            conn: db::init_conn()?,
            current_url_id: -1,
            domain: String::from("de.populus.wiki"),
            urls: Vec::new(),
            url_ids_received: Vec::new(),
        })
    }

    fn fill_urls(&mut self) -> Result<()> {
        let url_models =
            db::select_crawl_urls(&mut self.conn, &self.domain, &self.url_ids_received)?;
        for (u, d) in url_models {
            // todo: how does into() in rust work?
            self.urls.push((u.url_id, u.to_url(&d.name)));
            self.url_ids_received.push(u.url_id);
        }
        Ok(())
    }

    pub fn get_item(&mut self) -> Result<Option<UrlItem>> {
        if self.urls.is_empty() {
            self.fill_urls()?;
        }
        let Some((id, url_item)) = self
            .urls
            .pop()
            .map(|(id, url)| (id, UrlItem { i: vec![], url }))
        else {
            return Ok(None);
        };
        self.current_url_id = id;
        Ok(Some(url_item))
    }

    pub fn put_outlinks(&mut self, fr: &FetchResult, outlinks: &[Outlink]) -> Result<()> {
        db::insert_fetch_result(&mut self.conn, self.current_url_id, fr, outlinks)
    }
}
