use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use anyhow::Result;
use url::{ParseError, Url};
use crate::url_util::is_http_s;
use crate::crawler::{Inlink, Outlink};

pub(super) struct HtmlExtractor;

// todo: also extract img, script, style, ...

impl super::Extractor for HtmlExtractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>> {
        let document = Document::from(body_str);
        let a_nodes = document.find(Name("a").and(Attr("href", ())));

        let mut outlinks: Vec<Outlink> = Vec::new();
        for node in a_nodes {
            // todo url must be shorter than 2048 characters according to https://en.m.wikipedia.org/wiki/Sitemaps
            let Some(href) = node.attr("href") else {
                warn!("document.find should only return nodes with href attribute!");
                continue;
            };

            if let Some(mut url) = match Url::parse(href) {
                Ok(url) if is_http_s(&url) => Some(url),
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
                // TODO remove safeguard!!!
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
