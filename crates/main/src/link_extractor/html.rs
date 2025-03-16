use crate::crawler::{Context, Outlink};
use anyhow::Result;
use crawl::link;
use crawl::url_util::parse_outlink;
use html5ever::LocalName;
use html5ever::local_name;
use html5ever::tokenizer::{
    BufferQueue, CharacterTokens, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult,
    Tokenizer, TokenizerOpts,
};
use mediatype::MediaTypeBuf;
use std::cell::RefCell;
use url::Url;

pub(super) struct HtmlExtractor;

// todo: also extract img, script, style, ...
// todo: respect <base> tag also see https://codeberg.org/unobtanium/unobtanium/issues/3 https://w3c.github.io/webappsec-csp/#directive-base-uri

impl super::Extractor for HtmlExtractor {
    fn get_outlinks(&self, body_str: &str, base: &Url) -> Result<Vec<Outlink>> {
        let buffer = BufferQueue::default();
        buffer.push_back(body_str.into());
        let sink = OutlinksSink::new(base.clone());
        let tokenizer = Tokenizer::new(sink, TokenizerOpts::default());
        let _ = tokenizer.feed(&buffer);
        tokenizer.end();
        Ok(tokenizer.sink.outlinks.into_inner())
    }
}

struct OutlinksSink {
    pub base_url: Url,
    pub outlinks: RefCell<Vec<Outlink>>,
}

impl OutlinksSink {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            outlinks: RefCell::new(Vec::new()),
        }
    }

    fn maybe_push_outlink(&self, rel: &[String], url: Url, context: Context) {
        if link::can_follow(rel) {
            self.outlinks.borrow_mut().push(Outlink { url, context });
        }
    }

    fn handle_a(&self, tag: &Tag, href: &str) {
        if let Some(url) = parse_outlink(&self.base_url, href) {
            let (rel, _) = get_link_attrs(tag);
            let context = if rel.iter().any(|x| x == "next") {
                Context::Next
            } else {
                Context::Other
            };
            self.maybe_push_outlink(&rel, url, context);
        }
    }

    /// For the moment only extracts links to feeds
    fn handle_link(&self, tag: &Tag, href: &str) {
        if let Some(url) = parse_outlink(&self.base_url, href) {
            let (rel, content_type) = get_link_attrs(tag);
            if link::is_feed_link(&rel, &content_type) {
                self.maybe_push_outlink(&rel, url, Context::FeedLink);
            }
        }
    }
}

impl TokenSink for OutlinksSink {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(tag) => {
                match tag.kind {
                    #[allow(clippy::single_match)]
                    TagKind::StartTag => match tag.name {
                        local_name!("a") => {
                            if let Some(href) = get_attr(&tag, &local_name!("href")) {
                                self.handle_a(&tag, &href);
                            }
                        }
                        local_name!("link") => {
                            if let Some(href) = get_attr(&tag, &local_name!("href")) {
                                self.handle_link(&tag, &href);
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

fn get_attr(tag: &Tag, attr_name: &LocalName) -> Option<String> {
    tag.attrs
        .iter()
        .find(|a| a.name.local == *attr_name)
        .map(|a| a.value.to_string())
}

fn get_link_attrs(tag: &Tag) -> (Vec<String>, Option<MediaTypeBuf>) {
    let rel = link::split_words(get_attr(tag, &local_name!("rel")).as_deref());
    let content_type: Option<MediaTypeBuf> = if let Some(attr) = get_attr(tag, &local_name!("type"))
    {
        match attr.parse() {
            Ok(m) => Some(m),
            Err(e) => {
                debug!("Content Type Parse Error: {e:?}");
                None
            }
        }
    } else {
        None
    };
    (rel, content_type)
}
