use anyhow::Result;
use bytes::Bytes;
use chrono::prelude::*;
use reqwest::{header::HeaderMap, Client, Error, Response, StatusCode, Url};
use std::time::{Duration, Instant, SystemTime};
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt, BufWriter};
use uuid::Uuid;

pub struct Fetcher {
    archive_file: Option<BufWriter<File>>,
    archive_file_cnt: u32,
    archive_file_bytes_written: usize,
    client: Client,
}

pub struct FetchResult {
    pub body: Bytes,
    pub duration_ms: u128,
    pub start: SystemTime,
    pub status: StatusCode,
    pub url: Url,
}

impl FetchResult {
    pub fn body_str(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
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
        Fetcher {
            archive_file: None,
            archive_file_cnt: 0,
            archive_file_bytes_written: 0,
            client,
        }
    }

    // TODO implement option to specify size limit
    // Yandex limits robots.txt to 500 KB https://yandex.ru/support/webmaster/controlling-robot/robots-txt.html?lang=en
    // Sitemaps are limited to 50 MB
    pub async fn fetch(&mut self, url: Url) -> Result<FetchResult> {
        debug!("Fetching {url}");
        let start_systemtime = SystemTime::now();
        let start_instant = Instant::now();
        let result = self.client.get(url.clone()).send().await;
        let duration = start_instant.elapsed();
        let duration_ms = duration.as_millis();

        let response = result?;

        // debug!("Unable to fetch [{:?}] {url} - {err}", err.status());
        // maybe FetchError?
        // https://github.com/spyglass-search/netrunner/blob/main/src/lib/crawler.rs
        //return Err(FetchError::RequestError(err))

        debug!(
            "fetched with status {} in {duration_ms} ms: {url}",
            response.status()
        );
        let status = response.status();
        let url = response.url().clone();
        let headers = response.headers().clone();
        let body: Bytes = response.bytes().await?;

        let fr = FetchResult {
            body,
            duration_ms,
            start: start_systemtime,
            status,
            url,
        };
        self.write_to_archive(&fr, &headers).await?;
        Ok(fr)
    }

    async fn write_to_archive(&mut self, fr: &FetchResult, headers: &HeaderMap) -> Result<()> {
        if self.archive_file.is_none() {
            let filename = format!("/home/thk/tmp/archive_{}.warc", self.archive_file_cnt);
            debug!("Starting new warc file: {filename}");
            let tokio_file = File::create(filename).await?;
            self.archive_file_cnt += 1;
            self.archive_file = Some(BufWriter::new(tokio_file));
            // TODO start a new file with a warcinfo record
        }
        let writer: &mut BufWriter<File> = self.archive_file.as_mut().unwrap();
        let bytes_written = Self::write_record(writer, fr, headers).await?;
        self.archive_file_bytes_written += bytes_written;

        // TODO set to ~64MB?
        if self.archive_file_bytes_written > 1024 * 1024 {
            writer.get_mut().sync_all().await?;
            self.archive_file = None;
        }

        Ok(())
    }

    /// WARC 1.1 spec:
    /// <https://github.com/iipc/warc-specifications/blob/master/specifications/warc-format/warc-1.1-annotated/index.md>
    async fn write_record(
        w: &mut BufWriter<File>,
        fr: &FetchResult,
        headers: &HeaderMap,
    ) -> io::Result<usize> {
        let mut cnt = 0;
        let mut headers_bytes: Vec<u8> = Vec::new();

        for (k, v) in headers {
            headers_bytes.extend(k.to_string().as_bytes());
            headers_bytes.extend(b": ");
            headers_bytes.extend(v.as_bytes());
            headers_bytes.extend(b"\r\n");
        }
        headers_bytes.extend(b"\r\n");

        cnt += w.write(b"WARC/1.1\r\nWARC-Type: response\r\nContent-Type: application/http; msgtype=response\r\nWARC-Record-ID: ").await?;
        cnt += w
            .write(Uuid::new_v4().to_urn().to_string().as_bytes())
            .await?;
        cnt += w.write(b"\r\nWARC-Target-URI: ").await?;
        cnt += w.write(fr.url.to_string().as_bytes()).await?;
        cnt += w.write(b"\r\nContent-Length: ").await?;
        // TODO: check whether body.len() is correct!
        let content_length = headers_bytes.len() + fr.body.len();
        cnt += w.write(content_length.to_string().as_bytes()).await?;
        cnt += w.write(b"\r\nWARC-Date: ").await?;
        // TODO: check correct formatting of date!
        // WARC-Date fr.start UTC timestamp formatted according to [W3CDTF]
        let dt: DateTime<Utc> = fr.start.into();
        cnt += w
            .write(dt.to_rfc3339_opts(SecondsFormat::Secs, true).as_bytes())
            .await?;
        cnt += w.write(b"\r\n\r\n").await?;

        cnt += w.write(&headers_bytes).await?;
        cnt += w.write(fr.body.as_ref()).await?;
        cnt += w.write(b"\r\n\r\n").await?;
        w.flush().await?;

        Ok(cnt)
    }
}
