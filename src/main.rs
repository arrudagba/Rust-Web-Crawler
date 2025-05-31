use reqwest;
use scraper::{Html, Selector};
use std::collections::{HashSet};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    env_logger::init();

    // Target page to scrape
    let root = "https://crawler-test.com/";

    // Downloads HTML from the root page
    let html = get_html(root).await?;
    
    let mut links = Vec::new();
    get_links(&html, root, &mut links);

    let mut visited: HashSet<String> = HashSet::new();
    let mut to_visit: Vec<String> = links.clone();

    // Flat BFS list — processes links layer by layer
    while !to_visit.is_empty() {
        let current_layer = to_visit.clone();
        to_visit.clear();

        for link in current_layer {
            if visited.contains(&link) {
                continue;
            }

            match get_html(&link).await {
                Ok(html) => {
                    visited.insert(link.clone());
                    get_links(&html, &link, &mut to_visit);
                }
                Err(e) => log::error!("{:?}", e),
            }
        }
    }

    // Logs all visited links
    for link in visited.iter() {
        log::info!("{:?}", link);
    } 

    Ok(())
}

/// Downloads the HTML content from a given URL.
/// Example: get_html("https://example.com").await? -> "<html>...</html>"
async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let response = response.error_for_status()?; // Fails if the response is not 2xx

    let html = response.text().await?;
    Ok(html)
}

/// Resolves a relative or absolute URL into a fully qualified absolute URL based on a root URL.
/// Example: get_url("https://example.com/blog/", "../about.html") -> "https://example.com/about.html"
fn get_url(root: &str, sub: &str) -> String {
    if sub.starts_with("https://") || sub.starts_with("http://") {
        return sub.into(); // Already a full URL
    }

    // Try to resolve relative URL using the base; fallback to manual concatenation
    match Url::parse(root).and_then(|base| base.join(sub)) {
        Ok(resolved) => resolved.into(),
        Err(_) => {
            format!(
                "{}/{}",
                root.strip_suffix("/").unwrap_or(root),
                sub.strip_prefix("/").unwrap_or(sub)
            )
        }
    }
}

/// Extracts all anchor tag hrefs from HTML content and resolves them into absolute URLs.
/// Example: get_links("<a href=\"/index.html\">Home</a>", "https://example.com", &mut links)
fn get_links(html: &str, url: &str, results: &mut Vec<String>) {
    let fragment = Html::parse_fragment(&html);
    let selector = Selector::parse("a").unwrap();

    for element in fragment.select(&selector) {
        let text = element.value().attr("href");
        match text {
            Some(val) => {
                let absolute = get_url(url, val);
                if !results.contains(&absolute) {
                    results.push(absolute);
                }
            },
            None => (), // Skips anchor tags without href
        }
    }
}
