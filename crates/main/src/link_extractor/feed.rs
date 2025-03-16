use crate::crawler::{Context, Inlink, Outlink};
use anyhow::Result;
use feed_rs::{model::Entry, parser};
use url::{/*ParseError,*/ Url};

pub(super) struct FeedExtractor;

impl super::Extractor for FeedExtractor {
    fn get_outlinks(&self, body_str: &str, _base: &Url) -> Result<Vec<Outlink>> {
        let feed = parser::parse(body_str.as_bytes())?;
        Ok(feed.entries.iter().filter_map(entry_to_outlink).collect())
    }
}

fn entry_to_outlink(entry: &Entry) -> Option<Outlink> {
    for link in &entry.links {
        // todo: probably ditch feed-rs here and write our own parsers for ATOM, RSS and Webfeed
        return Some(Outlink{
            url: if let Ok(url) = Url::parse(link.href.as_str()) { url } else { return None },
            i: Inlink {
                context: Context::Feed,
                .. Inlink::default()
            }
        })
    }
    None
}
