use std::collections::HashMap;
use url::Url;

pub trait UrlFrontier {
    fn get_url(&mut self) -> Option<Url>;

    fn put_url(&mut self, url: Url);
    fn len(&self) -> usize;
}

#[allow(clippy::module_name_repetitions)]
pub struct UrlFrontierVec {
    urls: Vec<Url>,
    url_data: HashMap<String, Option<()>>,
}

impl UrlFrontierVec {
    pub fn new() -> Self {
        UrlFrontierVec {
            urls: vec![],
            url_data: HashMap::new(),
        }
    }
}

impl UrlFrontier for UrlFrontierVec {
    fn get_url(&mut self) -> Option<Url> {
        self.urls.pop()
    }

    fn put_url(&mut self, url: Url) {
        let url_string = url.to_string();
        if self.url_data.contains_key(&url_string) {
            return;
        }
        self.urls.push(url);
        self.url_data.insert(url_string, Some(()));
    }

    fn len(&self) -> usize {
        self.urls.len()
    }
}
