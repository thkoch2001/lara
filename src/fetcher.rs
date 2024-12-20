use reqwest::{Client, Error, Response, Url};
use std::time::{Duration, Instant, SystemTime};

pub struct Fetcher {
    client: Client,
}

pub struct FetchResult {
    pub duration_ms: u128,
    pub result: Result<Response, Error>,
    pub start: SystemTime,
}

impl Fetcher {
    pub fn new(bot_name: &str) -> Fetcher {
        // TODO Get URL from a better place, e.g. Cargo.toml?
        let ua_name = format!(
            "{bot_name}/{} https://github.com/thkoch2001/lara#larabot",
            env!("CARGO_PKG_VERSION")
        );
        let client = Client::builder()
            .user_agent(ua_name)
            .gzip(true)
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failure while building HTTP client");
        Fetcher { client }
    }

    // TODO implement option to specify size limit
    // Yandex limits robots.txt to 500 KB https://yandex.ru/support/webmaster/controlling-robot/robots-txt.html?lang=en
    // Sitemaps are limited to 50 MB
    pub async fn fetch(&self, url: Url) -> FetchResult {
        debug!("Fetching {url}");
        let start_systemtime = SystemTime::now();
        let start_instant = Instant::now();
        let result = self.client.get(url.clone()).send().await;
        let duration = start_instant.elapsed();
        let duration_ms = duration.as_millis();

        match result {
            Err(ref err) => {
                debug!("Unable to fetch [{:?}] {url} - {err}", err.status());
                // maybe FetchError?
                // https://github.com/spyglass-search/netrunner/blob/main/src/lib/crawler.rs
                //return Err(FetchError::RequestError(err))
            }
            Ok(ref response) => {
                debug!(
                    "fetched with status {} in {duration_ms} ms: {url}",
                    response.status()
                );
            }
        };
        FetchResult {
            duration_ms,
            result,
            start: start_systemtime,
        }
    }
}
