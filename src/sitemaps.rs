use crate::fetcher::Fetcher;
use crate::url_frontier::UrlFrontier;
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::str;
use url::Url;

pub fn run(
    url: &Url,
    sitemap_urls: &mut Vec<Url>,
    fetcher: &mut Fetcher,
    url_frontier: &mut UrlFrontier,
) -> Result<u32> {
    let mut urls_count = 0;
    if sitemap_urls.is_empty() {
        // theoretically, there could be sitemaps below an URL path
        let url =
            Url::parse(format!("{}://{}/sitemap.xml", url.scheme(), url.authority()).as_ref())?;
        sitemap_urls.push(url);
    }

    // TODO protect against infinite loops
    while !sitemap_urls.is_empty() {
        let fr = fetcher.fetch(&sitemap_urls.pop().expect("len>0"))?;
        let (mut sitemap_urls_found, urls_added) = parse(&fr.body_str(), url_frontier);
        sitemap_urls.append(&mut sitemap_urls_found);
        urls_count += urls_added;
    }
    Ok(urls_count)
}

fn parse(body_str: &str, url_frontier: &mut UrlFrontier) -> (Vec<Url>, u32) {
    // TODO can we get the body as a stream?
    // https://users.rust-lang.org/t/how-to-stream-reqwest-response-to-a-gzip-decoder/69706/4
    // TODO implement file size restriction of 50 MB
    // https://doc.rust-lang.org/stable/std/io/trait.Read.html#method.take
    let mut urls_count = 0;
    let mut reader = Reader::from_str(body_str);
    reader.config_mut().trim_text(true);
    let mut in_entry = false;
    let mut sitemap_urls: Vec<Url> = Vec::new();
    let mut entry: HashMap<String, String> = HashMap::new();
    let mut key: String = String::new();
    //let mut buf = Vec::new();

    loop {
        // TODO can we also use reader.read_event()?
        match reader.read_event() {
            //_into(&mut buf) {
            Err(e) => debug!("Error at position {}: {e:?}", reader.error_position()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"url" | b"sitemap" => in_entry = true,
                name => {
                    if in_entry {
                        if let Ok(s) = str::from_utf8(name) {
                            // Clippy made me write this.
                            <str as AsRef<str>>::as_ref(s).clone_into(&mut key);
                        }
                    }
                }
            },
            Ok(Event::Text(e)) => {
                if in_entry {
                    entry.insert(key.to_string(), e.unescape().unwrap().into_owned());
                }
            }
            Ok(Event::End(e)) => {
                if let name @ (b"url" | b"sitemap") = e.name().as_ref() {
                    in_entry = false;
                    if entry.contains_key("loc") {
                        if let Ok(url) = Url::parse(entry.get("loc").unwrap()) {
                            // TODO also use the other data, especially lastmod!
                            match name {
                                b"url" => {
                                    url_frontier.put_url(url);
                                    urls_count += 1;
                                }
                                b"sitemap" => sitemap_urls.push(url),
                                _ => panic!("We already matched on this before!"),
                            };
                        }
                    }
                }
            }
            _ => (),
        }
    }
    #[allow(unreachable_code)]
    (sitemap_urls, urls_count)
}
