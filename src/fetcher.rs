// There's a std: [Fetch](https://fetch.spec.whatwg.org). However it doesn't
// seem to apply to a crawler.

use crate::env_config::{ARCHIVE_DIR, FROM};
use anyhow::Result;
use chrono::prelude::*;
use flate2::{write::GzEncoder, Compression};
use http::{HeaderMap, StatusCode, Version};
use simple_moving_average::{NoSumSMA, SMA};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use ureq::config::AutoHeaderValue as AHV;
use ureq::tls::{TlsConfig, TlsProvider};
use ureq::Agent;
use url::Url;
use uuid::Uuid;

// TODO Move to configuration
/// Maximum size for HTTP response body
const MAX_BODY_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
const MIN_FETCH_DURATION: Duration = Duration::from_millis(150); // We actually wait 3xavg fetch duration between fetches

struct Politeness {
    until: SystemTime,
    duration_avg: NoSumSMA<Duration, u32, 10>,
}

impl Default for Politeness {
    fn default() -> Self {
        Politeness {
            until: SystemTime::UNIX_EPOCH,
            duration_avg: NoSumSMA::<Duration, u32, 10>::from_zero(Duration::ZERO),
        }
    }
}

impl Politeness {
    pub fn update(&mut self, fr: &FetchResult) {
        match fr.status.as_u16() {
            200 => {
                self.duration_avg
                    .add_sample(std::cmp::max(fr.duration, MIN_FETCH_DURATION));
                self.until = fr.start.add(self.duration_avg.get_average() * 3);
            }
            429 => todo!(),
            _ => (), // todo
        }
    }

    pub fn wait(&self) {
        crate::clock::wait(self.until);
    }
}

pub struct Fetcher {
    archive_dir: PathBuf,
    archive_file: Option<GzEncoder<File>>,
    archive_file_cnt: u32,
    archive_file_bytes_written: usize,
    agent: Agent,
    /// FROM header for requests
    from: String,
    politeness: HashMap<String, Politeness>,
}

impl Drop for Fetcher {
    fn drop(&mut self) {
        debug!("dropping fetcher");
        if self.archive_file.is_some() {
            if let Err(e) = self.close_archive_file() {
                error!("{e:?}");
            }
        }
    }
}

pub struct FetchResult {
    pub body: Vec<u8>, // TODO what's the advantage of Bytes crate?
    pub duration: Duration,
    pub start: SystemTime,
    pub status: StatusCode,
    pub http_version: Version,
}

impl FetchResult {
    pub fn body_str(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    fn status_line(&self) -> String {
        format!("{:?} {}\r\n", self.http_version, self.status)
    }
}

impl Fetcher {
    pub fn new(bot_name: &str) -> Fetcher {
        let archive_dir = ARCHIVE_DIR.parse::<PathBuf>();
        let m = archive_dir.metadata().unwrap_or_else(|_| {
            panic!(
                "Could not get metadata of ARCHIVE_DIR: {}",
                archive_dir.display()
            )
        });
        assert!(m.is_dir(), "Not a dir: {}", archive_dir.display());

        // TODO Get URL from a better place, e.g. Cargo.toml?
        let ua_name = format!(
            "{bot_name}/{} https://github.com/thkoch2001/lara#larabot",
            env!("CARGO_PKG_VERSION")
        );
        let agent = Agent::config_builder()
            .http_status_as_error(false)
            .user_agent(AHV::Provided(Arc::new(ua_name)))
            .max_redirects(0) // TODO: Handle redirects
            .timeout_connect(Some(Duration::from_millis(5000)))
            .timeout_global(Some(Duration::from_millis(20000)))
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    .build(),
            )
            .build()
            .into();
        Fetcher {
            archive_dir,
            archive_file: None,
            archive_file_cnt: 0, // TODO make this an option and search for last file of current job in case of NONE
            archive_file_bytes_written: 0,
            agent,
            from: FROM.get(),
            politeness: HashMap::new(),
        }
    }

    pub fn close_archive_file(&mut self) -> Result<()> {
        debug!("closing archive file");
        self.archive_file.as_mut().unwrap().try_finish()?;
        self.archive_file = None;
        self.archive_file_bytes_written = 0;
        Ok(())
    }

    // TODO implement option to specify size limit
    // Yandex limits robots.txt to 500 KB https://yandex.ru/support/webmaster/controlling-robot/robots-txt.html?lang=en
    // Sitemaps are limited to 50 MB
    pub fn fetch(&mut self, url: &Url) -> Result<FetchResult> {
        debug!("Fetching {url}");
        // todo clean old entries from politeness HashMap
        let politeness = self
            .politeness
            .entry(url.authority().to_string())
            .or_default();
        politeness.wait();
        let start_systemtime = SystemTime::now();
        let start_instant = Instant::now();
        let result = self
            .agent
            .get(url.to_string())
            .header("FROM", self.from.clone())
            .call();
        let duration = start_instant.elapsed();

        let mut response = result?;

        // debug!("Unable to fetch [{:?}] {url} - {err}", err.status());
        // maybe FetchError?
        // https://github.com/spyglass-search/netrunner/blob/main/src/lib/crawler.rs
        //return Err(FetchError::RequestError(err))

        debug!(
            "fetched with status {} in {} ms: {url}",
            response.status(),
            duration.as_millis()
        );
        let status = response.status();
        let http_version = response.version();
        let headers = response.headers().clone();
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_SIZE)
            .read_to_vec()?;

        let fr = FetchResult {
            body,
            duration,
            start: start_systemtime,
            status,
            http_version,
        };
        politeness.update(&fr);
        self.write_to_archive(url, &fr, &headers)?;
        Ok(fr)
    }

    // TODO: directly compress the archive file
    fn write_to_archive(&mut self, url: &Url, fr: &FetchResult, headers: &HeaderMap) -> Result<()> {
        if self.archive_file.is_none() {
            let path = format!(
                "{}/archive_{:03}.warc.gz",
                self.archive_dir.display(),
                self.archive_file_cnt
            );
            debug!("Starting new warc file: {path}");
            let file = File::create(path)?;
            let gzip_encoder = GzEncoder::new(file, Compression::best());
            self.archive_file_cnt += 1;
            self.archive_file = Some(gzip_encoder);
            // TODO start a new file with a warcinfo record
        }
        let writer = self.archive_file.as_mut().unwrap();
        let bytes_written = Self::write_record(writer, url, fr, headers)?;
        self.archive_file_bytes_written += bytes_written;

        // TODO somehow get the size of the compressed file?
        // file.metadata().unwrap().len() encoder has get_ref()
        // Optimization: check metadata only after at least the threshold of uncompressed bytes has been written
        if self.archive_file_bytes_written > 1024 * 1024 {
            let () = self.close_archive_file()?;
        }

        Ok(())
    }

    /// WARC 1.1 spec:
    /// <https://github.com/iipc/warc-specifications/blob/master/specifications/warc-format/warc-1.1-annotated/index.md>
    fn write_record(
        w: &mut GzEncoder<File>,
        url: &Url,
        fr: &FetchResult,
        headers: &HeaderMap,
    ) -> io::Result<usize> {
        let mut cnt = 0;
        let mut headers_bytes: Vec<u8> = Vec::new();

        headers_bytes.extend(fr.status_line().as_bytes());
        for (k, v) in headers {
            headers_bytes.extend(k.to_string().as_bytes());
            headers_bytes.extend(b": ");
            headers_bytes.extend(v.as_bytes());
            headers_bytes.extend(b"\r\n");
        }
        headers_bytes.extend(b"\r\n");

        cnt += w.write(b"WARC/1.1\r\nWARC-Type: response\r\nContent-Type: application/http; msgtype=response\r\nWARC-Record-ID: ")?;
        cnt += w.write(Uuid::new_v4().urn().to_string().as_bytes())?;
        cnt += w.write(b"\r\nWARC-Target-URI: ")?;
        cnt += w.write(url.to_string().as_bytes())?;
        cnt += w.write(b"\r\nContent-Length: ")?;
        // TODO: check whether body.len() is correct!
        let content_length = headers_bytes.len() + fr.body.len();
        cnt += w.write(content_length.to_string().as_bytes())?;
        cnt += w.write(b"\r\nWARC-Date: ")?;
        // TODO: check correct formatting of date!
        // WARC-Date fr.start UTC timestamp formatted according to [W3CDTF]
        let dt: DateTime<Utc> = fr.start.into();
        cnt += w.write(dt.to_rfc3339_opts(SecondsFormat::Secs, true).as_bytes())?;
        cnt += w.write(b"\r\n\r\n")?;

        cnt += w.write(&headers_bytes)?;
        cnt += w.write(fr.body.as_ref())?;
        cnt += w.write(b"\r\n\r\n")?;
        w.flush()?;

        Ok(cnt)
    }
}
