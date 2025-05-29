use reqwest;
use scraper::{Html, Selector};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    env_logger::init();

    // Target page to scrape
    let url = "";
    
    // Download HTML content from the page
    let html = get_html(url).await?; 

    let links = spyder(&html, url);
    match links {
        Ok(vec) => {
            for item in vec.iter() {
                log::info!("{:?}", item);
            }
        },
        Err(e) => log::error!("{:?}", e)
    }

    Ok(())
}

/// Downloads the HTML content from a given URL.
/// e.g., get_html("https://example.com").await? -> "<html>...</html>"
async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let response = response.error_for_status()?; // Fail on non-2xx HTTP status

    let html = response.text().await?;
    Ok(html)
}

/// Resolves a relative or absolute URL into a full absolute URL based on a root URL.
/// e.g., get_url("https://example.com/blog/", "../about.html") -> "https://example.com/about.html"
fn get_url(root: &str, sub: &str) -> String {
    if sub.starts_with("https://") || sub.starts_with("http://") {
        return sub.into(); // Already a full URL
    }

    // Try to join with base URL, fallback to manual concatenation
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
/// e.g., spyder("<a href=\"/index.html\">Home</a>", "https://example.com") -> Ok(["https://example.com/index.html"])
fn spyder(html: &str, url: &str) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
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
            None => (), // Skip anchors with no href
        }
    }

    Ok(results)
}
