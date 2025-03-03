//! TODO:
//! - <https://yandex.ru/support/webmaster/robot-workings/clean-param.html#clean-param>
//!
//! # Crawl rate
//!
//! - neither Yandex nor Google respect Crawl-Delay in robots.txt since it is mostly misconfigured. Sites should use HTTP Status 429 instead.
//!   <https://webmaster.yandex.ru/blog/skorost-obkhoda-ili-ob-izmeneniyakh-v-uchete-direktivy-crawl-delay>
//!
//! - <https://developers.google.com/search/docs/crawling-indexing/reduce-crawl-rate>

use crate::env_config::BOT_NAME;
use crate::fetcher::Fetcher;
use crate::link_extractor::extract_outlinks;
use crate::robotstxt::{CheckResult, RobotsTxt};
use crate::signal_handler::SignalHandler;
use crate::url_frontier::UrlFrontier;
use anyhow::Result;
use crawl::url_util::{is_domain_root, with_path_only};
use mediatype::MediaTypeBuf;
use url::Url;

pub struct Crawler {
    fetcher: Fetcher,
    robotstxt: RobotsTxt,
    signal_handler: SignalHandler,
    url_frontier: UrlFrontier,
}

/// These contexts are not the ones from [Mime Sniffing
/// standard](https://mimesniff.spec.whatwg.org) but should be possible to
/// map.
#[derive(diesel_derive_enum::DbEnum, Debug, Default, Clone)]
#[ExistingTypePath = "crate::db::schema::sql_types::UrlContext"]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum Context {
    // TODO: consistent naming, e.g. FromSitemap, FromFeed
    // TODO: rename to LinkContext and also in db?
    Seed,
    /// A link pointing to a sitemap, either found in robots.txt or the
    /// default location /sitemap.xml.
    IsSitemap,
    /// A link found in a sitemap
    Sitemap,
    /// A link pointing to a syndication feed (RSS, ATOM), probably found in a
    /// html HEAD LINK element:
    /// `<head><link rel="alternate" type="application/rss+xml" href="..." />`
    FeedLink,
    /// A link found in a feed
    Feed,
    #[default]
    Other,
    Img,
    Style,
    Script,
}

#[derive(Default, Clone)]
pub struct Inlink {
    /// TODO <https://en.wikipedia.org/wiki/Nofollow>
    /// or PREV/NEXT, however we're only interested in NEXT
    #[allow(dead_code)]
    pub rel: Vec<String>,
    pub context: Context,
    pub content_type: Option<MediaTypeBuf>,
}

pub struct Outlink {
    pub url: Url,
    pub i: Inlink,
}

pub struct UrlItem {
    pub url: Url,
    pub i: Vec<Inlink>,
}

impl Crawler {
    pub fn new(signal_handler: SignalHandler) -> Result<Self> {
        let bot_name = BOT_NAME.get();
        Ok(Self {
            fetcher: Fetcher::new(&bot_name.clone()),
            robotstxt: RobotsTxt::new(&bot_name),
            signal_handler,
            url_frontier: UrlFrontier::new()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let grace = self.signal_handler.grace();
        while let Some(item) = self.url_frontier.get_item()? {
            let url = &item.url;
            debug!("Crawl: '{url}'");
            match self.robotstxt.check(url, &mut self.fetcher)? {
                CheckResult::Allowed => (),
                CheckResult::Disallowed => {
                    info!("Crawling of {url} forbidden by robots.txt");
                    continue;
                }
                CheckResult::Retry(seconds) => {
                    info!("Retry robots check for {url} in {seconds}s");
                    todo!("put back url for retry in seconds");
                    //continue;
                }
            }

            let (fr, body) = self.fetcher.fetch(&item.url.clone())?;
            let mut outlinks = extract_outlinks(&item, &fr, &body)?;
            if is_domain_root(url) {
                debug!("Adding sitemap outlinks for domain root: {url}");
                let mut sitemap_outlinks: Vec<Outlink> = self
                    .robotstxt
                    .get_sitemaps(url, &mut self.fetcher)?
                    .into_iter()
                    .map(|url| Outlink {
                        url,
                        i: Inlink {
                            context: Context::Sitemap,
                            ..Inlink::default()
                        },
                    })
                    .collect();
                if sitemap_outlinks.is_empty() {
                    sitemap_outlinks = vec![Outlink {
                        url: with_path_only(url, "sitemap.xml"),
                        i: Inlink {
                            context: Context::Sitemap,
                            ..Inlink::default()
                        },
                    }];
                }
                outlinks.append(&mut sitemap_outlinks);
            }
            // TODO what to do with domains not crawled right now?
            // TODO this will probably often kick the currently crawled domain out of the robotstxt cache?
            // outlinks = self.robotstxt.filter_outlinks(outlinks, &mut self.fetcher);

            self.url_frontier.put_outlinks(&fr, &outlinks)?;

            if grace.is_interrupted() {
                break;
            }
        }
        Ok(())
    }
}
