mod cache;

use anyhow::Result;
use cache::{AccessResult as AR, Cache};
use reqwest::Url;
use std::rc::Rc;
use std::time::{Duration, SystemTime};
use texting_robots::Robot;

pub struct Manager {
    cache: Cache<Robot>,
    bot_name: String,
}

#[derive(PartialEq)]
pub enum CheckResult {
    Allowed,
    Disallowed,
    /// Come back later in n seconds
    Retry(i32),
}

impl Manager {
    pub fn new(bot_name: &str) -> Manager {
        Self {
            cache: Cache::<Robot>::new(),
            bot_name: String::from(bot_name),
        }
    }

    pub async fn check(&self, url: &Url) -> Result<CheckResult> {
        let ar = self.get_or_fetch(url).await?;

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

    async fn get_or_fetch(&self, url: &Url) -> Result<AR<Robot>> {
        let authority = url.authority();

        let mut unreachable_first_tried: Option<SystemTime> = None;
        if let Some(r) = self.cache.get(authority) {
            let ar = &r.ar;
            let updated = r.updated;
            match ar {
                AR::Unavailable | AR::Ok(_) if !elapsed(&updated, ONE_DAY) => return Ok(ar.clone()),
                AR::Unreachable(first_tried) => {
                    // TODO: replace with exponential backoff
                    if !elapsed(&updated, ONE_DAY) {
                        return Ok(ar.clone());
                    }
                    if elapsed(&updated, 30 * ONE_DAY) {
                        return Ok(AR::Unavailable);
                    }
                    unreachable_first_tried = Some(*first_tried);
                }
                _ => (),
            };
        }

        let scheme = url.scheme();
        let robots_url = format!("{scheme}://{authority}/robots.txt");
        info!("Fetching {robots_url}");
        let response = reqwest::get(robots_url).await?;
        let ar = match response.status().as_u16() {
            400..=499 => AR::Unavailable,
            200 => {
                let body = response.text().await?;
                let robot = Robot::new(&self.bot_name, body.as_bytes());
                AR::Ok(Rc::new(robot.unwrap()))
            }
            _ => AR::Unreachable(unreachable_first_tried.unwrap_or(SystemTime::now())),
        };

        self.cache.insert(authority, ar.clone());
        Ok(ar)
    }
}

const ONE_DAY: u64 = 24 * 60 * 60;

fn elapsed(since: &SystemTime, seconds: u64) -> bool {
    let duration = since.elapsed().unwrap_or(Duration::from_secs(1));
    duration.as_secs() > seconds
}
