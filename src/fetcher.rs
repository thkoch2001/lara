// This code was written with lines copied from https://github.com/spyglass-search/netrunner/blob/main/src/lib/crawler.rs

use reqwest::{Client, /*StatusCode, */Url, Response};
use std::time::Duration;

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new(bot_name: String) -> Fetcher {
        let ua_name = format!("{bot_name}/{}", env!("CARGO_PKG_VERSION"));
        let client = Client::builder().user_agent(ua_name)
            .gzip(true)
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failure while building HTTP client");
        Fetcher { client }
    }

    pub async fn fetch(&self, url: Url) -> anyhow::Result<Response, reqwest::Error> {
        let result = self.client.get(url.clone()).send().await;
        if let Err(err) = result {
            log::warn!("Unable to fetch [{:?}] {} - {}", err.status(), url, err);
            //return Err(FetchError::RequestError(err))
            return Err(err)
        }
        Ok(result.unwrap())
    }
}

