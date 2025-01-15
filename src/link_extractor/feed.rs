use crate::crawler::{Inlink, Outlink};
use crate::url_util::is_http_s;
use anyhow::Result;
use url::{ParseError, Url};

pub(super) struct FeedExtractor;

// todo: also extract img, script, style, ...

impl super::Extractor for FeedExtractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>> {
        debug!("todo: Feed extractor not yet done.");
        Ok(Vec::new())
    }
}
