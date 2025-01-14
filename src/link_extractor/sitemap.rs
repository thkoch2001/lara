use crate::crawler::{Context, Inlink, Outlink};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::str;
use url::Url;

use anyhow::Result;

pub(super) struct SitemapExtractor;

impl super::Extractor for SitemapExtractor {
    fn get_outlinks(&self, body_str: &str, _: &Url) -> Result<Vec<Outlink>> {
        // TODO can we get the body as a stream?
        // https://users.rust-lang.org/t/how-to-stream-reqwest-response-to-a-gzip-decoder/69706/4
        // TODO implement file size restriction of 50 MB
        // https://doc.rust-lang.org/stable/std/io/trait.Read.html#method.take

        let mut reader = Reader::from_str(body_str);
        reader.config_mut().trim_text(true);
        let mut in_entry = false;
        let mut outlinks: Vec<Outlink> = Vec::new();
        let mut entry: HashMap<String, String> = HashMap::new();
        let mut key: String = String::new();

        loop {
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
                    if let entry_name @ (b"url" | b"sitemap") = e.name().as_ref() {
                        in_entry = false;
                        let context = match entry_name {
                            b"url" => Context::Other,
                            b"sitemap" => Context::Sitemap,
                            _ => panic!("We already matched on this before!"),
                        };

                        if let Some(outlink) = entry_to_outlink(&entry, context) {
                            // todo can use append here with Option?
                            outlinks.push(outlink);
                        }
                    }
                }
                _ => (),
            }
        }
        #[allow(unreachable_code)]
        Ok(outlinks)
    }
}
fn entry_to_outlink(entry: &HashMap<String, String>, context: Context) -> Option<Outlink> {
    if !entry.contains_key("loc") {
        return None;
    }

    let Ok(url) = Url::parse(entry.get("loc").unwrap()) else {
        return None;
    };

    // TODO also use the other data, especially lastmod!

    Some(Outlink {
        url,
        i: Inlink {
            context,
            ..Inlink::default()
        },
    })
}
