use crate::env_config::DB_URL;
use anyhow::Result;
use diesel::prelude::*;
use url::Url;

pub mod models;
pub mod schema;

#[derive(QueryableByName, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct Id(
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(column_name = id)]
    i32,
);

pub fn format_bind_params(rows: usize, columns: usize) -> String {
    (0..rows)
        .map(|row| {
            format!(
                "({})",
                ((row * columns + 1)..=((row + 1) * columns))
                    .map(|column| format!("${column}"))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        })
        .collect::<Vec<String>>()
        .join(",")
}

/// Query idea found here:
/// <https://dba.stackexchange.com/questions/46410/how-do-i-insert-a-row-which-contains-a-foreign-key>
pub fn insert_urls(conn: &mut PgConnection, urls: &Vec<Url>) -> Result<usize> {
    use diesel::sql_types::{Nullable, Text};

    let q_str = format!(
        include_str!("insert_urls.sql"),
        format_bind_params(urls.len(), 3)
    );

    let mut q = diesel::sql_query(q_str).into_boxed();
    for url in urls {
        q = q
            .bind::<Text, _>(url.domain().unwrap().to_string())
            .bind::<Text, _>(url.path().to_string())
            .bind::<Nullable<Text>, _>(url.query().map(String::from));
    }
    let affected = q.execute(conn)?;
    debug!("insert_urls got {} urls, inserted {affected}", urls.len());
    Ok(affected)
}

pub fn select_crawl_urls(
    conn: &mut PgConnection,
    domain: &str,
    exclude_url_ids: &Vec<i32>,
) -> Result<Vec<(models::Url, models::Domain)>> {
    use crate::db::schema::domain::dsl;
    use crate::db::schema::url::dsl::{url, url_id};
    use diesel::dsl::not;

    Ok(url
        .inner_join(dsl::domain)
        .filter(dsl::name.eq(domain))
        .filter(not(url_id.eq_any(exclude_url_ids)))
        .select((models::Url::as_select(), models::Domain::as_select()))
        .limit(10)
        .load(conn)?)
}

pub fn init_conn() -> Result<PgConnection> {
    Ok(PgConnection::establish(&DB_URL.get())?)
}
