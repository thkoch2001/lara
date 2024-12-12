mod cache;

use anyhow::Result;
use cache::{AccessResult, Cache, Entry};
use reqwest::Url;
use std::rc::Rc;
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
        let rc = self.get_or_fetch(url).await?;
        let ar = &rc.ar;
        let _updated = rc.updated;

        Ok(match ar {
            AccessResult::Unavailable => CheckResult::Allowed,
            AccessResult::Unreachable => CheckResult::Retry(84000),
            AccessResult::Ok(robot) => {
                if robot.allowed(url.as_ref()) {
                    CheckResult::Allowed
                } else {
                    CheckResult::Disallowed
                }
            }
        })
    }

    async fn get_or_fetch(&self, url: &Url) -> Result<Rc<Entry<Robot>>> {
        let authority = url.authority();

        if let Some(r) = self.cache.get(authority) {
            return Ok(r);
        }

        let scheme = url.scheme();
        let robots_url = format!("{scheme}://{authority}/robots.txt");
        info!("Fetching {robots_url}");
        let response = reqwest::get(robots_url).await?;
        let ar = match response.status().as_u16() {
            400..=499 => AccessResult::Unavailable,
            200 => {
                let body = response.text().await?;
                let robot = Robot::new(&self.bot_name, body.as_bytes());
                AccessResult::Ok(robot.unwrap())
            }
            _ => AccessResult::Unreachable,
        };

        Ok(self.cache.insert_clone(authority, ar))
    }
}
