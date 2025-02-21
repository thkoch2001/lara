// Relevant:
// * MDN: [X-Content-Type-Options](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options)
// * MDN: [Mime Types](https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types)
// * [Mime Sniffing standard](https://mimesniff.spec.whatwg.org)

use crate::crawler::{Context, Inlink, Outlink, UrlItem};
use crate::fetcher::FetchResult;
use anyhow::Result;
use feed::FeedExtractor;
use html::HtmlExtractor;
use itertools::Itertools;
use mime::{HTML, TEXT, XML};
use sitemap::SitemapExtractor;
use std::any::type_name_of_val;
use url::Url;

mod feed;
mod html;
mod sitemap;

pub fn extract_outlinks(item: &UrlItem, fr: &FetchResult) -> Result<Vec<Outlink>> {
    let inlink = get_inlink(&item.i);
    // todo: also extract links from headers: Feeds, pagination next, script, style, ...
    let extractor = get_extractor(fr, &inlink);
    let mut outlinks = extractor.get_outlinks(fr.body_str().as_ref(), &item.url)?;
    // Doesn't seem worthwile to look for links in HTTP header
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Link
    outlinks = outlinks.into_iter().unique_by(|o| o.url.clone()).collect();
    debug!(
        "{}: {} outlinks from {}",
        extractor.name(),
        outlinks.len(),
        &item.url
    );
    Ok(outlinks)
}

fn get_inlink(links: &[Inlink]) -> Inlink {
    // todo: get the best inlink, not the first
    if links.is_empty() {
        return Inlink::default();
    }
    links[0].clone()
}

fn get_extractor(fr: &FetchResult, inlink: &Inlink) -> Box<dyn Extractor> {
    match fr.parse_content_type() {
        Err(e) => debug!("{e:?}"),
        Ok(mime) => match (mime.type_(), mime.subtype()) {
            (TEXT, HTML) => return Box::new(HtmlExtractor),
            (_, XML) if SitemapExtractor::is(&fr.body) => return Box::new(SitemapExtractor),
            _ => (),
        },
    }

    match inlink.context {
        Context::Other => Box::new(HtmlExtractor),
        Context::Sitemap => Box::new(SitemapExtractor),
        Context::Feed => Box::new(FeedExtractor),
        _ => todo!(),
    }
}

pub trait Extractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>>;

    fn name(&self) -> &'static str {
        let full_name = &type_name_of_val(&self);
        debug_assert!(full_name[..32] == *"&lara::link_extractor::sitemap::");
        &full_name[32..]
    }
}
