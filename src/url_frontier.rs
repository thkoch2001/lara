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

pub struct UrlFrontier {
    urls: Vec<Url>,
    url_data: HashMap<String, Option<()>>,
}

impl UrlFrontier {
    pub fn new() -> Self {
        UrlFrontier {
            urls: vec![],
            url_data: HashMap::new(),
        }
    }

    pub fn get_url(&mut self) -> Option<Url> {
        self.urls.pop()
    }

    pub fn put_url(&mut self, url: Url) {
        let url_string = url.to_string();
        if self.url_data.contains_key(&url_string) {
            return;
        }
        self.urls.push(url);
        self.url_data.insert(url_string, Some(()));
    }
}
