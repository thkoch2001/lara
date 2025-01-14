// Relevant:
// * MDN: [X-Content-Type-Options](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options)
// * MDN: [Mime Types](https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types)
// * [Mime Sniffing standard](https://mimesniff.spec.whatwg.org)

use crate::crawler::{Context, Inlink, Outlink, UrlItem};
use crate::fetcher::FetchResult;
use crate::sitemaps::SitemapExtractor;

use anyhow::Result;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use url::{ParseError, Url};

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

struct HtmlExtractor;

impl Extractor for HtmlExtractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>> {
        let document = Document::from(body_str);
        let a_nodes = document.find(Name("a").and(Attr("href", ())));

        let mut outlinks: Vec<Outlink> = Vec::new();
        for node in a_nodes {
            // We already filtered for a nodes with href attribute
            // url must be shorter than 2048 characters according to https://en.m.wikipedia.org/wiki/Sitemaps
            let href = node.attr("href").unwrap();

            if let Some(mut url) = match Url::parse(href) {
                Ok(url) if url.scheme() == "http" || url.scheme() == "https" => Some(url),
                Ok(_) => None,
                Err(ParseError::RelativeUrlWithoutBase) => match base.join(href) {
                    Ok(url) => Some(url),
                    Err(err) => {
                        debug!("{:?}: {href}", err);
                        None
                    }
                },
                Err(err) => {
                    debug!("{:?}: {href}", err);
                    None
                }
            } {
                if url.fragment().is_some() {
                    url.set_fragment(None);
                }
                if url.to_string() == base.to_string() {
                    continue;
                }
                // TODO remove
                if url.host_str() != Some("de.populus.wiki") {
                    continue;
                }
                outlinks.push(Outlink {
                    url,
                    i: Inlink {
                        rel: node.attr("rel").map(std::string::ToString::to_string),
                        ..Inlink::default()
                    },
                });
            }
        }
        Ok(outlinks)
    }
}
