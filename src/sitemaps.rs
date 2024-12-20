use crate::fetcher::Fetcher;
use crate::url_frontier::{UrlFrontier, UrlFrontierVec};
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Url;
use std::collections::HashMap;
use std::str;

pub async fn run(url: Url, sitemap_urls: &mut Vec<Url>, fetcher: &Fetcher, url_frontier: &mut UrlFrontierVec) -> Result<u32> {
    let mut urls_count = 0;
    if sitemap_urls.len() == 0 {
        // theoretically, there could be sitemaps below an URL path
        let url = Url::parse(format!("{}://{}/sitemap.xml", url.scheme(), url.authority()).as_ref())?;
        sitemap_urls.push(url);
    }

    // TODO protect against infinite loops
    while sitemap_urls.len() > 0 {
        match fetcher.fetch(sitemap_urls.pop().expect("len>0")).await {
            Err(err) => debug!("{:?}", err),
            Ok(response) => {
                let body = response.text().await?;
                let (mut sitemap_urls_found, urls_added) = parse(&body, url_frontier);
                sitemap_urls.append(&mut sitemap_urls_found);
                urls_count += urls_added;
            }
        }
    }
    Ok(urls_count)
}

fn parse(body: &str, url_frontier: &mut UrlFrontierVec) -> (Vec<Url>, u32){
    // TODO can we get the body as a stream?
    // https://users.rust-lang.org/t/how-to-stream-reqwest-response-to-a-gzip-decoder/69706/4
    // TODO implement file size restriction of 50 MB
    // https://doc.rust-lang.org/stable/std/io/trait.Read.html#method.take
    let mut urls_count = 0;
    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);
    let mut in_entry = false;
    let mut sitemap_urls: Vec<Url> = Vec::new();
    let mut entry: HashMap<String, String> = HashMap::new();
    let mut key: String = "".to_string();
    //let mut buf = Vec::new();

    loop {

        // TODO can we also use reader.read_event()?
        match reader.read_event() {//_into(&mut buf) {
            Err(e) => debug!("Error at position {}: {e:?}", reader.error_position()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"url" | b"sitemap" => in_entry = true,
                name => if in_entry {
                    if let Ok(s) = str::from_utf8(name) {
                        key = <str as AsRef<str>>::as_ref(s).to_owned();
                    }
                },
            },
            Ok(Event::Text(e)) => if in_entry {
                   entry.insert(key.to_string(), e.unescape().unwrap().into_owned());
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                name@(b"url" | b"sitemap") => {
                    in_entry = false;
                    if entry.contains_key("loc") {
                        if let Ok(url) = Url::parse(entry.get("loc").unwrap()) {
                            match name {
                                b"url" => {
                                    url_frontier.put_url(url);
                                    urls_count += 1;
                                },
                                b"sitemap" => sitemap_urls.push(url),
                                _ => panic!("We already matched on this before!"),
                            };
                        }
                    }
                },
                _ => ()
            },

            _ => (),
        }
        //buf.clear();
    }
    #[allow(unreachable_code)]
    (sitemap_urls, urls_count)
}
