use crate::crawler::{Context, Outlink};
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
    #[allow(clippy::never_loop)]
    for link in &entry.links {
        // todo: probably ditch feed-rs here and write our own parsers for ATOM, RSS and Webfeed
        //
        // ATOM: https://www.rfc-editor.org/rfc/rfc4287
        // atom:entry elements that contain no child atom:content element
        // MUST contain at least one atom:link element with a rel attribute
        // value of "alternate".
        //   o  atom:entry elements MUST NOT contain more than one atom:link
        //      element with a rel attribute value of "alternate" that has the
        //      same combination of type and hreflang attribute values.
        //   o  atom:entry elements MAY contain additional atom:link elements
        //      beyond those described above.
        //
        // Webfeed: https://www.jsonfeed.org/version/1.1/
        // item->url
        //
        // RSS: https://www.rssboard.org/rss-specification
        // just the link element
        return Some(Outlink {
            url: if let Ok(url) = Url::parse(link.href.as_str()) {
                url
            } else {
                return None;
            },
            context: Context::Feed,
        });
    }
    None
}
