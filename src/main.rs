use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::collections::{HashSet};
use reqwest;
use std::error::Error;
use env_logger::{Builder, Target};
use scraper::{Html, Selector};
use url::Url;
use serde::Serialize;

enum OutputFormat {
    PlainText(File),
    Json(File),
}

struct Config {
    root_url: String,
    depth: i32,
    verbose: bool,
    response_error: bool,
    output_file: Option<OutputFormat>,
}

#[derive(Serialize)]
struct CrawlOutput<'a> {
    root: &'a str,
    found_urls: Vec<&'a str>,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    Builder::from_env(env_logger::Env::default().default_filter_or("info")).target(Target::Stderr).init();

    let config = parse_args().expect("Failed to parse arguments");

    let html = get_html(&config.root_url).await?;
    let mut links = Vec::new();
    get_links(&html, &config.root_url, &mut links);

    let mut visited: HashSet<String> = HashSet::new();
    let mut to_visit: Vec<String> = links.clone();
    let mut error_links: Vec<String> = Vec::new();
    
    // Main crawling loop
    while !to_visit.is_empty() {
        let current_layer = to_visit.clone();
        to_visit.clear();

        for link in current_layer {
            // Skip if the link had a request error
            if error_links.contains(&link){
                visited.remove(&link);
                continue;
            }

            // Prints any link except already printed
            if !visited.contains(&link){
                if config.verbose{
                        log::info!("{:?}", &link);
                }
            }

            // Skip if already visited, from another domain, or over depth limit
            if visited.contains(&link) || !is_same_domain(&config.root_url, &link) || depth_control(&link, config.depth) {
                visited.insert(link);
                continue;
            }

            // Try to fetch HTML and extract links
            match get_html(&link).await {
                Ok(html) => {
                    visited.insert(link.clone());
                    get_links(&html, &link, &mut to_visit);
                }
                Err(e) => {
                    error_links.push(link.clone());
                    visited.remove(&link);
                    log::error!("{:?}", format_reqwest_error(&e));
                    // Print error if user sets arg "-e"
                    if config.response_error{
                        log::info!("{:?}", &link);
                    }
                },
            }
        }
    }

    // Handle output writing based on the selected format
    if let Some(output) = config.output_file {
        match output {
            OutputFormat::PlainText(mut file) => {
                for link in visited.iter() {
                    writeln!(file, "{}", link).expect("Failed to write to file");
                }
            }
            OutputFormat::Json(mut file) => {
                let output = CrawlOutput {
                    root: &config.root_url,
                    found_urls: visited.iter().map(|s| s.as_str()).collect(),
                };
                let json = serde_json::to_string_pretty(&output).expect("Failed to serialize JSON");
                file.write_all(json.as_bytes()).expect("Failed to write JSON to file");
            }
        }
    }

    // Add response error URLs if arg "-e" is set by user 
    if config.response_error{
        for item in error_links{
            visited.insert(item);
        }
    }

    if !config.verbose {
        // Print all logs first
        for link in visited.iter() {
            log::info!("{:?}", link);
        }
        
        println!("Crawling completed!");
    }
    else{
        println!("Crawling completed!");
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

/// Checks if a candidate url has the same host then the root url.
/// e.g., is_same_domain("https://example.com/", "https://example2.com/") -> false
fn is_same_domain(root: &str, candidate: &str) -> bool {
    let root_host = url::Url::parse(root).ok().and_then(|u| u.host_str().map(|h| h.to_string()));
    let candidate_host = url::Url::parse(candidate).ok().and_then(|u| u.host_str().map(|h| h.to_string()));

    root_host == candidate_host
} 

/// Checks the depth of the current url being listed has the same depth that the user wants to visit.
/// e.g., depth_control("https://example.com/1/2/", 2) -> false
fn depth_control(url: &str, depth: i32) -> bool{
    // If depth = 0, it means that the user don't want a depth control
    if depth == 0 {return false;}

    let parsed_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false, 
    };

    // Count the segments
    let count = parsed_url
        .path_segments()
        .map(|segments| segments.filter(|s| !s.is_empty()).count() as i32)
        .unwrap_or(-1); 

    if count == depth{
        return true;
    }

    false
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

/// Parses command-line arguments and returns the configuration for the crawler.
/// Supports flags for crawl depth, output file (plain or JSON), display error URLs, verbose and help.
fn parse_args() -> Result<Config, io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Missing parameters.\nUsage: ./web_crawler [URL] [-d DEPTH] [-f [FILENAME]]\nTry './web_crawler -h' for more information.");
        std::process::exit(1);
    }

    if matches!(args[1].as_str(), "-h" | "--h" | "-help" | "--help") {
        println!(
            "Usage: web_crawler [options] <url>\n\
            \n\
            Options:\n\
            \t-d, --depth <n>              Limit the crawl depth (default: 0)\n\
            \t-f, --file [filename]        Write visited URLs to file (default: output.txt)\n\
            \t-fj, --file-json [filename]  Write visited URLs to JSON file (default: output.json)\n\
            \t-e, --request-error          Display/Save the URLs that have returned error in the request(default: disabled)\n\
            \t-v, --verbose                Enable verbose logging during the crawl.\n\
            \t-h, --help                   Display this help message and exit\n\
            \n\
            Examples:\n\
            \tweb_crawler https://example.com\n\
            \tweb_crawler https://example.com -d 2\n\
            \tweb_crawler https://example.com -f\n\
            \tweb_crawler https://example.com -f results.txt -d 3\n\
            \tweb_crawler https://example.com -fj results.json\n\
            \tweb_crawler https://example.com -e\n\
            \tweb_crawler https://example.com -v\n\
            \n\
            This tool crawls a website starting from the provided URL, collecting internal links recursively.\n\
            Use depth to limit the recursion, and file to save the results."
        );
        std::process::exit(0);
    }

    let root_url = args[1].clone();
    let mut depth: i32 = 0;
    let mut verbose: bool = false;
    let mut response_error: bool = false;
    let mut output_file: Option<OutputFormat> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--d" => {
                if i + 1 >= args.len() {
                    eprintln!("Expected value after {}", args[i]);
                    std::process::exit(1);
                }
                depth = args[i + 1].parse::<i32>().unwrap_or_else(|_| {
                    eprintln!("Invalid depth value");
                    std::process::exit(1);
                });
                i += 2;
            }
            "-f" | "--f" => {
                let filename = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    args[i].clone()
                } else {
                    "output.txt".to_string()
                };
                output_file = Some(OutputFormat::PlainText(File::create(&filename)?));
                i += 1;
            }
            "-fj" | "--fj" => {
                let filename = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    args[i].clone()
                } else {
                    "output.json".to_string()
                };
                output_file = Some(OutputFormat::Json(File::create(&filename)?));
                i += 1;
            }
            "-v" | "--v" =>{
                verbose = true;
                i += 1;
            }
            "-e" | "--e" =>{
                response_error = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    Ok(Config {
        root_url,
        depth,
        verbose,
        response_error,
        output_file,
    })
}

/// Format the reqwest::Error so it can be readable
/// e.g., format_reqwet_error("reqwest::Error { kind: Status(404), url: "https://example.com/" }") -> "HTTP error: 404 | URL: https://example.com/"
fn format_reqwest_error(e: &reqwest::Error) -> String {
    let mut msg = String::new();

    if let Some(status) = e.status() {
        msg.push_str(&format!("HTTP error: {}", status));
    } else if e.is_timeout() {
        msg.push_str("Timeout error");
    } else if e.is_connect() {
        msg.push_str("Connection error");
    } else if e.is_request() {
        msg.push_str("Request error");
    } else {
        msg.push_str("Unknown error");
    }

    if let Some(url) = e.url() {
        msg.push_str(&format!(" | URL: {}", url));
    }

    if let Some(source) = e.source() {
        msg.push_str(&format!(" | Caused by: {}", source));
    }

    msg
}
