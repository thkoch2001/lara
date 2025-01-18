use crate::url_util;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = crate::db::schema::domain)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Domain {
    domain_id: i32,
    pub name: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = crate::db::schema::url)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Domain, foreign_key = domain_id))]
pub struct Url {
    pub url_id: i32,
    pub domain_id: i32,
    pub path: String,
    pub query: Option<String>,
}

impl Url {
    pub fn to_url(&self, domain_name: &str) -> url::Url {
        url_util::build(domain_name, &self.path, self.query.as_deref())
    }
}
