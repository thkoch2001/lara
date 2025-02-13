use crate::crawler::{Inlink, Outlink};
use crate::url_util::parse_outlink;
use anyhow::Result;
use html5ever::local_name;
use html5ever::tokenizer::{
    BufferQueue, CharacterTokens, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult,
    Tokenizer, TokenizerOpts,
};
use html5ever::LocalName;
use url::Url;

pub(super) struct HtmlExtractor;

// todo: also extract link, img, script, style, ...
// todo: respect <base> tag also see https://codeberg.org/unobtanium/unobtanium/issues/3 https://w3c.github.io/webappsec-csp/#directive-base-uri

impl super::Extractor for HtmlExtractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>> {
        let mut buffer = BufferQueue::new();
        buffer.push_back(body_str.into());
        let sink = OutlinksSink::new(base.clone());
        let mut tokenizer = Tokenizer::new(sink, TokenizerOpts::default());
        let _ = tokenizer.feed(&mut buffer);
        tokenizer.end();
        Ok(tokenizer.sink.outlinks)
    }
}

struct OutlinksSink {
    pub base_url: Url,
    pub outlinks: Vec<Outlink>,
}

impl OutlinksSink {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            outlinks: Vec::new(),
        }
    }

    fn handle_a(&mut self, tag: &Tag, href: &str) {
        if let Some(url) = parse_outlink(&self.base_url, &href.to_string()) {
            self.outlinks.push(Outlink {
                url,
                i: Inlink {
                    rel: get_attr(tag, local_name!("rel")),
                    ..Inlink::default()
                },
            });
        }
    }
}

impl TokenSink for OutlinksSink {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(tag) => {
                match tag.kind {
                    TagKind::StartTag => match tag.name {
                        local_name!("a") => {
                            if let Some(href) = get_attr(&tag, local_name!("href")) {
                                self.handle_a(&tag, &href);
                            }
                        }
                        _ => (),
                    },
                    TagKind::EndTag => {
                        // todo: end collecting link text
                    }
                }
            }
            CharacterTokens(_str) => {
                // todo: ignore links without any text in them?
            }
            _ => (),
        }

        TokenSinkResult::Continue
    }
}

fn get_attr(tag: &Tag, attr_name: LocalName) -> Option<String> {
    tag.attrs
        .iter()
        .find(|&a| a.name.local == attr_name)
        .map(|a| a.value.to_string())
}
