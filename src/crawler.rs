//! TODO:
//! - <https://yandex.ru/support/webmaster/robot-workings/clean-param.html#clean-param>
//!
//! # Crawl rate
//!
//! - neither Yandex nor Google respect Crawl-Delay in robots.txt since it is mostly misconfigured. Sites should use HTTP Status 429 instead.
//!   <https://webmaster.yandex.ru/blog/skorost-obkhoda-ili-ob-izmeneniyakh-v-uchete-direktivy-crawl-delay>
//!
//! - <https://developers.google.com/search/docs/crawling-indexing/reduce-crawl-rate>

use crate::clock;
use crate::env_config::BOT_NAME;
use crate::fetcher::Fetcher;
use crate::robotstxt_cache::{AccessResult as AR, Cache as RobotsTxtCache};
use crate::url_frontier::UrlFrontier;

use crate::signal_handler::SignalHandler;
use crate::link_extractor::extract_outlinks;
use crate::url_util::{is_domain_root, with_path_only};
use anyhow::Result;
use std::rc::Rc;
use std::time::SystemTime;
use texting_robots::Robot;
use url::Url;

#[derive(PartialEq)]
pub enum CheckResult {
    Allowed,
    Disallowed,
    /// Come back later in n seconds
    Retry(i32),
}

pub struct Crawler {
    bot_name: String,
    fetcher: Fetcher,
    robotstxt_cache: RobotsTxtCache<Robot>,
    signal_handler: SignalHandler,
    url_frontier: UrlFrontier,
}

/// These contexts are not the ones from [Mime Sniffing
/// standard](https://mimesniff.spec.whatwg.org) but should be possible to
/// map.
#[derive(Default, Clone)]
pub enum Context {
    #[default]
    Other,
    Img,
    Style,
    Script,
    /// e.g. <head><link rel="alternate" type="application/rss+xml" href="..." />
    Feed,
    /// implicit from domain root or from robots.txt
    Sitemap,
}

#[derive(Default, Clone)]
pub struct Inlink {
    /// TODO <https://en.wikipedia.org/wiki/Nofollow>
    /// or PREV/NEXT, however we're only interested in NEXT
    pub rel: Option<String>,
    pub context: Context,
    pub redirect_count: usize,
    pub content_type: Option<String>,
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
    pub fn new(signal_handler: SignalHandler) -> Self {
        let bot_name = BOT_NAME.get();
        Crawler {
            bot_name: bot_name.clone(),
            fetcher: Fetcher::new(&bot_name),
            robotstxt_cache: RobotsTxtCache::new(SystemTime::now()),
            signal_handler,
            url_frontier: UrlFrontier::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // todo
        self.url_frontier.put_outlink(&Outlink {
            url: Url::parse("https://de.populus.wiki")?,
            i: Inlink::default(),
        });
        let grace = self.signal_handler.grace();
        while let Some(item) = self.url_frontier.get_item() {
            let url = &item.url;
            match self.check_robotstxt(url)? {
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

            let fr = self.fetcher.fetch(&item.url.clone())?;
            let mut outlinks = extract_outlinks(&item, &fr)?;
            debug!("extracted {} outlinks from {url}", outlinks.len());
            if is_domain_root(url) {
                debug!("Adding sitemap outlinks for domain root: {url}");
                let mut sitemap_outlinks = self.get_sitemaps_from_robotstxt(url)?;
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
            let outlinks = self.robotstxt_filter_outlinks(outlinks);

            self.url_frontier.put_outlinks(&item.url, &outlinks);

            if grace.is_interrupted() {
                break;
            }
        }
        Ok(())
    }

    // robots.txt stuff below
    pub fn get_sitemaps_from_robotstxt(&mut self, url: &Url) -> Result<Vec<Outlink>> {
        let mut outlinks: Vec<Outlink> = Vec::new();
        if let AR::Ok(robot) = self.get_or_fetch_robotstxt(url)? {
            for sitemap_url in &robot.sitemaps {
                if let Ok(url) = Url::parse(sitemap_url) {
                    outlinks.push(Outlink {
                        url,
                        i: Inlink {
                            context: Context::Sitemap,
                            ..Inlink::default()
                        },
                    });
                }
            }
        }
        Ok(outlinks)
    }

    fn robotstxt_filter_outlinks(&mut self, outlinks: Vec<Outlink>) -> Vec<Outlink> {
        outlinks
            .into_iter()
            .filter(|o| match self.check_robotstxt(&o.url) {
                Ok(CheckResult::Allowed) => true,
                Err(e) => {
                    info!("Error while filtering outlink {} {e:?}", &o.url);
                    false
                }
                _ => false,
            })
            .collect()
    }

    pub fn check_robotstxt(&mut self, url: &Url) -> Result<CheckResult> {
        let ar = self.get_or_fetch_robotstxt(url)?;

        Ok(match ar {
            AR::Unavailable => CheckResult::Allowed,
            AR::Unreachable(_first_tried) => CheckResult::Retry(84000),
            AR::Ok(robot) => {
                if robot.allowed(url.as_ref()) {
                    CheckResult::Allowed
                } else {
                    CheckResult::Disallowed
                }
            }
        })
    }

    fn get_or_fetch_robotstxt(&mut self, url: &Url) -> Result<AR<Robot>> {
        let authority = url.authority();

        let mut unreachable_first_tried: Option<SystemTime> = None;
        if let Some(r) = self.robotstxt_cache.get(authority) {
            let ar = &r.ar;
            let updated = r.updated;
            match ar {
                AR::Unavailable | AR::Ok(_) if !clock::elapsed(updated, clock::ONE_DAY) => {
                    return Ok(ar.clone())
                }
                AR::Unreachable(first_tried) => {
                    // TODO: replace with exponential backoff
                    if !clock::elapsed(updated, clock::ONE_DAY) {
                        return Ok(ar.clone());
                    }
                    if clock::elapsed(updated, 30 * clock::ONE_DAY) {
                        return Ok(AR::Unavailable);
                    }
                    unreachable_first_tried = Some(*first_tried);
                }
                _ => (),
            };
        }

        let robots_url = with_path_only(url, "robots.txt");
        let fetchresult = self.fetcher.fetch(&robots_url)?;

        let ar = match fetchresult.status.as_u16() {
            400..=499 => AR::Unavailable,
            200 => {
                let robot = Robot::new(&self.bot_name, &fetchresult.body);
                AR::Ok(Rc::new(robot.unwrap()))
            }
            _ => AR::Unreachable(unreachable_first_tried.unwrap_or(fetchresult.start)),
        };

        self.robotstxt_cache
            .insert(authority, ar.clone(), fetchresult.start);
        Ok(ar)
    }
}
