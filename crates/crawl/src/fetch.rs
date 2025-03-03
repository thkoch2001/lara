use anyhow::{Result, anyhow};
use chrono::prelude::*;
use http::header::CONTENT_TYPE;
use http::{HeaderMap, HeaderValue, StatusCode, Version};
use mediatype::MediaTypeBuf;
use std::time::Duration;

pub struct FetchResult {
    pub duration: Duration,
    pub headers: HeaderMap<HeaderValue>,
    pub start: DateTime<Utc>,
    pub status: StatusCode,
    pub http_version: Version,
}

impl FetchResult {
    /// Workaround to have this line for the warc archive although we don't get it from ureq.
    pub fn status_line(&self) -> String {
        format!("{:?} {}\r\n", self.http_version, self.status)
    }

    pub fn parse_content_type(&self) -> Result<MediaTypeBuf> {
        if let Some(content_type) = self.headers.get(CONTENT_TYPE) {
            Ok(content_type.to_str()?.parse()?)
        } else {
            Err(anyhow!("No Content_Type header."))
        }
    }
}
