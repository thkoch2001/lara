// Relevant:
// * MDN: [X-Content-Type-Options](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options)
// * MDN: [Mime Types](https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types)
// * [Mime Sniffing standard](https://mimesniff.spec.whatwg.org)

use crate::crawler::{Context, Inlink, Outlink, UrlItem};
use crate::fetcher::FetchResult;
use sitemap::SitemapExtractor;
use html::HtmlExtractor;

use anyhow::Result;
use url::Url;

mod sitemap;
mod html;

pub fn extract_outlinks(item: &UrlItem, fr: &FetchResult) -> Result<Vec<Outlink>> {
    let inlink = get_inlink(&item.i);
    // todo: also extract links from headers: Feeds, pagination next, script, style, ...
    get_extractor(fr, &inlink).get_outlinks(fr.body_str().as_ref(), &item.url)
}

fn get_inlink(links: &[Inlink]) -> Inlink {
    // todo: get the best inlink, not the first
    if links.is_empty() {
        return Inlink::default();
    }
    links[0].clone()
}

fn get_extractor(_fr: &FetchResult, inlink: &Inlink) -> Box<dyn Extractor> {
    match inlink.context {
        Context::Other => Box::new(HtmlExtractor),
        Context::Sitemap => Box::new(SitemapExtractor),
        _ => todo!(),
    }
}

pub trait Extractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>>;
}
