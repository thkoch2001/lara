use crate::clock::TimeToDrop;
use crate::crawler::Outlink;
use crate::env_config::DB_URL;
use crate::fetcher::FetchResult;
use anyhow::Result;
use diesel::prelude::*;
use std::time::Duration;

pub mod models;
#[allow(clippy::wildcard_imports)]
pub mod schema;

fn format_bind_params_row<T: Iterator<Item = usize>>(r: T) -> String {
    r.map(|i: usize| format!("${i}"))
        .collect::<Vec<String>>()
        .join(",")
}

fn format_bind_params(rows: usize, columns: usize) -> String {
    (0..rows)
        .map(|row| {
            format!(
                "({})",
                format_bind_params_row((row * columns + 1)..=((row + 1) * columns))
            )
        })
        .collect::<Vec<String>>()
        .join(",")
}

pub fn insert_fetch_result(
    conn: &mut PgConnection,
    url_id: i32,
    fr: &FetchResult,
    outlinks: &[Outlink],
) -> Result<()> {
    if outlinks.is_empty() {
        insert_fr(conn, url_id, fr)
    } else {
        insert_fr_with_urls(conn, url_id, fr, outlinks)
    }
}

fn insert_fr(conn: &mut PgConnection, p_url_id: i32, fr: &FetchResult) -> Result<()> {
    use self::schema::fetch_result::dsl::{fetch_result, http_status, time, url_id};

    diesel::insert_into(fetch_result)
        .values(&(
            url_id.eq(p_url_id),
            time.eq(fr.start),
            http_status.eq(i16::try_from(fr.status.as_u16()).unwrap()),
        ))
        .execute(conn)?;
    Ok(())
}

/// Query idea found here:
/// <https://dba.stackexchange.com/questions/46410/how-do-i-insert-a-row-which-contains-a-foreign-key>
fn insert_fr_with_urls(
    conn: &mut PgConnection,
    url_id: i32,
    fr: &FetchResult,
    outlinks: &[Outlink],
) -> Result<()> {
    use diesel::sql_types::{Integer, Nullable, SmallInt, Text, Timestamptz};
    let _ttd = TimeToDrop::new("insert_fr_with_urls", Duration::from_millis(1000));

    let outlinks_len = outlinks.len();

    let q_str = format!(
        include_str!("insert_urls.sql"),
        format_bind_params(outlinks_len, 4),
        format_bind_params_row(outlinks_len * 4 + 1..outlinks_len * 4 + 4)
    );

    let mut q = diesel::sql_query(q_str).into_boxed();
    for outlink in outlinks {
        let url = &outlink.url;
        q = q
            .bind::<Text, _>(url.domain().unwrap().to_string())
            .bind::<Text, _>(url.path().to_string())
            .bind::<Nullable<Text>, _>(url.query().map(String::from))
            .bind::<self::schema::sql_types::UrlContext, _>(&outlink.i.context);
    }

    q = q
        .bind::<Integer, _>(url_id)
        .bind::<Timestamptz, _>(fr.start)
        .bind::<SmallInt, _>(i16::try_from(fr.status.as_u16()).unwrap());

    let _ = q.execute(conn)?;
    Ok(())
}

pub fn select_crawl_urls(
    conn: &mut PgConnection,
    domain_name: &str,
    exclude_url_ids: &Vec<i32>,
) -> Result<Vec<(models::Url, models::Domain)>> {
    use crate::db::schema::domain::dsl::{domain, name};
    use crate::db::schema::url::dsl::{url, url_id};
    use diesel::dsl::not;

    Ok(url
        .inner_join(domain)
        .filter(name.eq(domain_name))
        .filter(not(url_id.eq_any(exclude_url_ids)))
        .select((models::Url::as_select(), models::Domain::as_select()))
        .limit(10)
        .load(conn)?)
}

pub fn init_conn() -> Result<PgConnection> {
    Ok(PgConnection::establish(&DB_URL.get())?)
}
