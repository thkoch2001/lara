use crate::clock;
use crate::fetcher::Fetcher;
use crate::robotstxt_cache::{AccessResult as AR, Cache as RobotsTxtCache};
use crate::sitemaps;
use crate::url_frontier::{UrlFrontier, UrlFrontierVec};
use anyhow::Result;
use reqwest::Url;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use std::rc::Rc;
use std::time::SystemTime;
use texting_robots::Robot;
use url::ParseError;

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
    url_frontier: UrlFrontierVec,
}

struct Outlink {
    // TODO https://en.wikipedia.org/wiki/Nofollow
    rel: Option<String>,
    url: Url,
}

impl Crawler {
    pub fn new(bot_name: &str) -> Self {
        Crawler {
            bot_name: bot_name.to_string(),
            fetcher: Fetcher::new(bot_name),
            robotstxt_cache: RobotsTxtCache::new(SystemTime::now()),
            url_frontier: UrlFrontierVec::new(),
        }
    }

    pub async fn run(&mut self, url: Url) -> Result<()> {
        let mut robotstxt_sitemaps = self.get_sitemaps_from_robotstxt(&url).await?;
        let urls_from_sitemaps_count = sitemaps::run(url, &mut robotstxt_sitemaps, &self.fetcher, &mut self.url_frontier).await?;
        debug!("Urls found from sitemaps: {urls_from_sitemaps_count}");

        while let Some(url) = self.url_frontier.get_url() {
            match self.check_robotstxt(&url).await? {
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

            let response = self.fetcher.fetch(url.clone()).await?;
            let body = response.text().await?;
            let document = Document::from(body.as_ref());
            let outlinks = find_outlinks(&document, &url);
            for outlink in outlinks {
                if urls_from_sitemaps_count == 0 {
                    self.url_frontier.put_url(outlink.url);
                }
            }
        }
        Ok(())
    }

    // robots.txt stuff below
    pub async fn get_sitemaps_from_robotstxt(&mut self, url: &Url) -> Result<Vec<Url>> {
        let mut sitemap_urls: Vec<Url> = Vec::new();
        Ok(match self.get_or_fetch_robotstxt(url).await? {
            AR::Ok(robot) => {
                for sitemap_url in &robot.sitemaps {
                    if let Ok(url) = Url::parse(&sitemap_url) {
                        sitemap_urls.push(url);
                    }
                }
                sitemap_urls
            },
            // TODO some more error handling?
            // - set crawl status to retry on 5xx error?
            _ => sitemap_urls,
        })
    }

    pub async fn check_robotstxt(&mut self, url: &Url) -> Result<CheckResult> {
        let ar = self.get_or_fetch_robotstxt(url).await?;

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

    async fn get_or_fetch_robotstxt(&mut self, url: &Url) -> Result<AR<Robot>> {
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

        let scheme = url.scheme();
        let robots_url = Url::parse(format!("{scheme}://{authority}/robots.txt").as_ref())?;
        let response = self.fetcher.fetch(robots_url).await?;
        debug!("Done Fetching robots.txt with status {}", response.status());
        // TODO: don't call SystemTime::now() twice. Reuse time from future FetchResult!
        let ar = match response.status().as_u16() {
            400..=499 => AR::Unavailable,
            200 => {
                let body = response.text().await?;
                let robot = Robot::new(&self.bot_name, body.as_bytes());
                AR::Ok(Rc::new(robot.unwrap()))
            }
            _ => AR::Unreachable(unreachable_first_tried.unwrap_or(SystemTime::now())),
        };

        self.robotstxt_cache
            .insert(authority, ar.clone(), SystemTime::now());
        Ok(ar)
    }
}

fn find_outlinks(document: &Document, base: &Url) -> Vec<Outlink> {
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
                rel: node.attr("rel").map(std::string::ToString::to_string),
            });
        }
    }
    outlinks
}