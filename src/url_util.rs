use url::Url;

pub fn is_domain_root(url: &Url) -> bool {
    (url.path() == "" || url.path() == "/") && url.query().is_none() && url.has_authority()
}

pub fn with_path_only(url: &Url, path: &str) -> Url {
    Url::parse(format!("{}://{}/{path}", url.scheme(), url.authority()).as_ref())
        .expect("Started from a valid URL")
}
