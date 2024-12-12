use reqwest::Url;

pub trait UrlFrontier {
    fn get_url(&mut self) -> Option<Url>;

    fn put_url(&mut self, url: Url);
}

#[allow(clippy::module_name_repetitions)]
pub struct UrlFrontierVec {
    urls: Vec<Url>,
}

impl UrlFrontierVec {
    pub fn new() -> Self {
        UrlFrontierVec { urls: vec![] }
    }
}

impl UrlFrontier for UrlFrontierVec {
    fn get_url(&mut self) -> Option<Url> {
        self.urls.pop()
    }

    fn put_url(&mut self, url: Url) {
        self.urls.push(url);
    }
}
