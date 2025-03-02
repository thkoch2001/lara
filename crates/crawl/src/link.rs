use mime::{APPLICATION, JSON, Mime, XML};

/// Parse attribute (e.g. 'rel') with space separated words.
///
/// Google [permits comma-separated rel-lists](google-rel), so we must expect
/// to see them too.
///
/// [mdn-rel]: https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/rel
/// [google-rel]: https://developers.google.com/search/docs/crawling-indexing/qualify-outbound-links
///
/// ```
/// # use crawl::link::split_words;
/// let a = split_words(Some(" UGC, noFollow  sponsored , "));
/// assert!(a == vec!["ugc", "nofollow", "sponsored"]);
///
/// let b = split_words(None);
/// assert!(b.is_empty());
/// ```
pub fn split_words(opt: Option<&str>) -> Vec<String> {
    if let Some(s) = opt {
        s.split(&[',', ' '][..])
            .filter(|x| !x.is_empty())
            .map(str::to_lowercase)
            .collect()
    } else {
        Vec::new()
    }
}

/// ```
/// # use crawl::link::is_feed_link;
/// # use mime::Mime;
/// let rels = &vec!["alternate".to_string()];
/// assert!(is_feed_link(
///   rels,
///   "application/rss+xml".parse().ok()
/// ));
/// assert!(is_feed_link(
///   rels,
///   "application/atom+xml".parse().ok()
/// ));
/// assert!(is_feed_link(
///   rels,
///   "application/feed+json".parse().ok()
/// ));
/// assert!(is_feed_link(
///   rels,
///   "Application/Atom+XML".parse().ok()
/// ));
/// assert!(!is_feed_link(
///   &vec![],
///   "application/atom+xml".parse().ok()
/// ));
/// assert!(!is_feed_link(
///   rels,
///   "application/atom".parse().ok()
/// ));
/// assert!(!is_feed_link(
///   rels,
///   "text/atom+xml".parse().ok()
/// ));
/// ```
pub fn is_feed_link(rels: &[String], content_type: &Option<Mime>) -> bool {
    if let Some(m) = content_type {
        return is_feed_type(m) && rels.iter().any(|x| x == "alternate");
    }
    false
}

/// ```
/// # use crawl::link::is_feed_type;
/// # use mime::Mime;
/// assert!(is_feed_type("application/rss+xml".parse().unwrap()));
/// assert!(is_feed_type("application/atom+xml".parse().unwrap()));
/// assert!(is_feed_type("application/feed+json".parse().unwrap()));
/// assert!(!is_feed_type("text/plain".parse().unwrap()));
/// ```
pub fn is_feed_type(m: &Mime) -> bool {
    m.type_() == APPLICATION
        && (m.suffix() == Some(XML) && (m.subtype() == "rss" || m.subtype() == "atom")
            || m.suffix() == Some(JSON) && m.subtype() == "feed")
}
