//! Bounded RSS 2.0 / Atom headline ingestion, with no HTTP requests or HTML rendering.
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub const MAX_NEWS_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_NEWS_ITEMS: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewsItem {
    pub source_id: String,
    pub title: String,
    pub url: String,
    pub published: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NewsKind { Rss, Atom, LicensedApi }

#[derive(Debug, Clone, Copy, Serialize)]
pub struct NewsProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub endpoint: &'static str,
    pub kind: NewsKind,
    pub enabled_by_default: bool,
}

pub const NEWS_PROVIDERS: &[NewsProvider] = &[
    NewsProvider { id: "bbc-top", name: "BBC News", endpoint: "https://feeds.bbci.co.uk/news/rss.xml", kind: NewsKind::Rss, enabled_by_default: false },
    NewsProvider { id: "npr-news", name: "NPR News", endpoint: "https://feeds.npr.org/1001/rss.xml", kind: NewsKind::Rss, enabled_by_default: false },
    NewsProvider { id: "guardian-world", name: "The Guardian World", endpoint: "https://www.theguardian.com/world/rss", kind: NewsKind::Rss, enabled_by_default: false },
];

#[derive(Debug, Error)]
pub enum NewsError {
    #[error("feed exceeds byte limit")]
    TooLarge,
    #[error("feed XML is invalid: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("feed is not RSS or Atom")]
    UnsupportedFormat,
}

fn direct_text(node: roxmltree::Node<'_, '_>) -> Option<String> {
    let value = node.children().filter_map(|n| n.text()).collect::<String>();
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty() { None } else { Some(value.chars().take(500).collect()) }
}
fn child_text(node: roxmltree::Node<'_, '_>, name: &str) -> Option<String> {
    node.children().find(|n| n.is_element() && n.tag_name().name() == name).and_then(direct_text)
}
fn secure_article_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url.trim()).ok()?;
    if parsed.scheme() != "https" || parsed.host_str().is_none()
       || !parsed.username().is_empty() || parsed.password().is_some() {
        return None;
    }
    Some(parsed.as_str().to_owned())
}

/// Parse an already downloaded RSS/Atom response. Article links are for display/navigation only,
/// never feed discovery, iframe creation, or automatic media connection.
pub fn parse_news(source_id: &str, body: &[u8]) -> Result<Vec<NewsItem>, NewsError> {
    if body.len() > MAX_NEWS_BYTES { return Err(NewsError::TooLarge); }
    let xml = std::str::from_utf8(body).map_err(|_| NewsError::UnsupportedFormat)?;
    let doc = roxmltree::Document::parse(xml)?;
    let root = doc.root_element();
    let atom = root.tag_name().name() == "feed" && root.tag_name().namespace() == Some("http://www.w3.org/2005/Atom");
    let rss = root.tag_name().name() == "rss";
    if !atom && !rss { return Err(NewsError::UnsupportedFormat); }
    let entries = root.descendants().filter(|n| n.is_element() && n.tag_name().name() == if atom { "entry" } else { "item" });
    let mut items = Vec::new();
    for entry in entries.take(MAX_NEWS_ITEMS) {
        let Some(title) = child_text(entry, "title") else { continue; };
        let link = if atom {
            entry.children().find(|n| n.is_element() && n.tag_name().name() == "link"
                && matches!(n.attribute("rel"), None | Some("alternate")))
                .and_then(|n| n.attribute("href")).map(str::to_owned)
        } else { child_text(entry, "link") };
        let Some(url) = link.as_deref().and_then(secure_article_url) else { continue; };
        let published = child_text(entry, if atom { "updated" } else { "pubDate" });
        let summary = child_text(entry, if atom { "summary" } else { "description" });
        items.push(NewsItem { source_id: source_id.chars().take(64).collect(), title, url, published, summary });
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rss_headline_and_url() {
        let rss = br#"<rss version="2.0"><channel><item><title>Road report</title><link>https://news.example/road</link><pubDate>Mon</pubDate></item></channel></rss>"#;
        let items = parse_news("test", rss).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Road report");
    }
    #[test]
    fn atom_alternate_link() {
        let atom = br#"<feed xmlns="http://www.w3.org/2005/Atom"><entry><title>Update</title><link rel="alternate" href="https://news.example/entry"/></entry></feed>"#;
        assert_eq!(parse_news("atom", atom).unwrap().len(), 1);
    }
    #[test]
    fn rejects_http_and_script_urls() {
        let rss = br#"<rss><channel><item><title>Bad</title><link>http://internal.local/</link></item><item><title>Bad2</title><link>javascript:alert(1)</link></item></channel></rss>"#;
        assert!(parse_news("test", rss).unwrap().is_empty());
    }
    #[test]
    fn rejects_oversized_or_invalid() {
        assert!(matches!(parse_news("test", &vec![0; MAX_NEWS_BYTES + 1]), Err(NewsError::TooLarge)));
        assert!(parse_news("test", b"<foo/>").is_err());
    }
}
