use crate::crawler::{Inlink, Outlink};
use crate::url_util::parse_outlink;
use anyhow::Result;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use url::Url;

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
            if let Some(url) = parse_outlink(base, &href) {
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
