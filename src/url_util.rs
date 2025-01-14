use url::Url;

/// Used by crawler to give special treatment to main site of a domain
pub fn is_domain_root(url: &Url) -> bool {
    (url.path() == "" || url.path() == "/") && url.query().is_none() && url.has_authority()
}

/// Used to create old style well known URLs (see examples below), not to be
/// confused with well known URIs from [RFC 8615][]
///
/// Examples:
///
/// * robots.txt http://www.robotstxt.org
/// * sitemap.xml https://www.sitemaps.org
/// * favicon.ico
///
/// [RFC 8615]: https://www.rfc-editor.org/rfc/rfc8615.html
pub fn with_path_only(url: &Url, path: &str) -> Url {
    Url::parse(format!("{}://{}/{path}", url.scheme(), url.authority()).as_ref())
        .expect("Started from a valid URL")
}

/// Used by HTML link extractor to only collect links to HTTP resources
pub fn is_http_s(url: &Url) -> bool {
    url.scheme() == "http" || url.scheme() == "https"
}
