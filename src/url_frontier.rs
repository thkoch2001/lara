//! URL frontier tells crawlers which URL to crawl next and eliminates duplicate
//! URLs.
//!
//! See also:
//! - [URL Frontier chapter][itir] in Introduction to Information Retrieval
//! - [Crawler-Commons URL frontier](https://github.com/crawler-commons/url-frontier)
//!
//! [itir]: https://nlp.stanford.edu/IR-book/html/htmledition/the-url-frontier-1.html

use std::collections::HashMap;
use url::Url;

use crate::crawler::{Inlink, Outlink, UrlItem};

pub struct UrlFrontier {
    urls: Vec<Url>,
    url_data: HashMap<String, Inlink>,
}

impl UrlFrontier {
    pub fn new() -> Self {
        UrlFrontier {
            urls: vec![],
            url_data: HashMap::new(),
        }
    }

    pub fn get_item(&mut self) -> Option<UrlItem> {
        self.urls.pop().map(|url| UrlItem {
            i: vec![self.url_data.get(&url.to_string()).unwrap().clone()],
            url,
        })
    }

    pub fn put_outlinks(&mut self, _url: &Url, outlinks: &Vec<Outlink>) {
        for outlink in outlinks {
            self.put_outlink(outlink);
        }
    }

    pub fn put_outlink(&mut self, outlink: &Outlink) {
        let url_string = outlink.url.to_string();
        if self.url_data.contains_key(&url_string) {
            return;
        }
        self.urls.push(outlink.url.clone());
        self.url_data.insert(url_string, outlink.i.clone());
    }
}
