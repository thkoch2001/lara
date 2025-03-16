// Relevant:
// * MDN: [X-Content-Type-Options](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options)
// * MDN: [Mime Types](https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types)
// * [Mime Sniffing standard](https://mimesniff.spec.whatwg.org)

use crate::crawler::Outlink;
use anyhow::{Result, bail};
use crawl::fetch::FetchResult;
use crawl::link;
use feed::FeedExtractor;
use html::HtmlExtractor;
use itertools::Itertools;
use mediatype::MediaTypeBuf;
use sitemap::SitemapExtractor;
use std::any::type_name_of_val;
use url::Url;

mod feed;
mod html;
mod sitemap;

pub fn extract_outlinks(url: &Url, fr: &FetchResult, body: &[u8]) -> Result<Vec<Outlink>> {
    // todo: also extract links from headers: Feeds, pagination next, script, style, ...
    let content_type = fr.parse_content_type()?;
    let extractor = get_extractor(&content_type, body)?;
    let mut outlinks = extractor.get_outlinks(&String::from_utf8_lossy(body), url)?;
    // Doesn't seem worthwile to look for links in HTTP header
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Link
    outlinks = outlinks.into_iter().unique_by(|o| o.url.clone()).collect();
    debug!(
        "{}: {} outlinks from {}",
        extractor.name(),
        outlinks.len(),
        &url
    );
    Ok(outlinks)
}

fn get_extractor(ct: &MediaTypeBuf, body: &[u8]) -> Result<Box<dyn Extractor>> {
    Ok(match (ct.ty().as_str(), ct.subty().as_str()) {
        ("text", "html") => Box::new(HtmlExtractor),
        (_, "xml") if SitemapExtractor::is(body) => Box::new(SitemapExtractor),
        (_, "xml") => Box::new(FeedExtractor),
        ("application", _) if link::is_feed_type(ct) => Box::new(FeedExtractor),
        _ => bail!("No extractor for {ct:?}"),
    })
}

pub trait Extractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>>;

    fn name(&self) -> &'static str {
        let full_name = &type_name_of_val(&self);
        debug_assert!(full_name[..32] == *"&lara::link_extractor::sitemap::");
        &full_name[32..]
    }
}
