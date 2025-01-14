use crate::clock;
use crate::crawler::{Context, Inlink, Outlink};
use crate::fetcher::Fetcher;
use crate::url_util::with_path_only;
use anyhow::Result;
use cache::{AccessResult as AR, Cache as RobotsTxtCache};
use std::rc::Rc;
use std::time::SystemTime;
use texting_robots::Robot;
use url::Url;

mod cache;

pub(super) struct RobotsTxt {
    robotstxt_cache: RobotsTxtCache<Robot>,
    bot_name: String,
}

#[derive(PartialEq)]
pub(super) enum CheckResult {
    Allowed,
    Disallowed,
    /// Come back later in n seconds
    Retry(i32),
}

impl RobotsTxt {
    pub fn new(bot_name: &str) -> Self {
        Self {
            bot_name: bot_name.to_string(),
            robotstxt_cache: RobotsTxtCache::new(SystemTime::now()),
        }
    }

    pub fn get_sitemaps(&mut self, url: &Url, fetcher: &mut Fetcher) -> Result<Vec<Outlink>> {
        let mut outlinks: Vec<Outlink> = Vec::new();
        if let AR::Ok(robot) = self.get_or_fetch_robotstxt(url, fetcher)? {
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

    pub fn filter_outlinks(
        &mut self,
        outlinks: Vec<Outlink>,
        fetcher: &mut Fetcher,
    ) -> Vec<Outlink> {
        outlinks
            .into_iter()
            .filter(|o| match self.check(&o.url, fetcher) {
                Ok(CheckResult::Allowed) => true,
                Err(e) => {
                    info!("Error while filtering outlink {} {e:?}", &o.url);
                    false
                }
                // todo maybe return true if robotstxt could not be retrieved right now?
                _ => false,
            })
            .collect()
    }

    pub fn check(&mut self, url: &Url, fetcher: &mut Fetcher) -> Result<CheckResult> {
        let ar = self.get_or_fetch_robotstxt(url, fetcher)?;

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

    fn get_or_fetch_robotstxt(&mut self, url: &Url, fetcher: &mut Fetcher) -> Result<AR<Robot>> {
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
        let fetchresult = fetcher.fetch(&robots_url)?;

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
