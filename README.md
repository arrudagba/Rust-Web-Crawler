# Rust Web Crawler

A simple, linearized BFS-based web crawler written in Rust.

This tool recursively fetches and extracts links from a given root URL within the same domain. It supports output to plain text or JSON, depth-limited traversal, error tracking, and verbose logging.

## Overview

This project implements a basic web crawler using a synchronous breadth-first search (BFS) strategy. It avoids concurrency for simplicity and predictable control over link traversal.

The crawler only follows links within the same domain as the root URL and stops at a user-specified depth.

## Features

- `-d`, `--depth <n>`: Limit the crawl depth (default: 0)
- `-f`, `--file [filename]`: Write visited URLs to file (default: output.txt)
- `-fj`, `--file-json [filename]`: Write visited URLs to JSON file (default: output.json)
- `-e`, `--request-error`: Display/Save URLs that returned request errors (default: disabled)
- `-v`, `--verbose`: Enable verbose logging during the crawl
- `-h`, `--help`: Display this help message and exit

## Dependencies

This project uses the following crates:

- `reqwest` – HTTP client
- `scraper` – HTML parser and CSS selectors
- `url` – URL parsing and domain handling
- `env_logger` – Logging system
- `serde` – JSON serialization
- `tokio` – asynchronous runtime (required by `reqwest`)

## Build & Run Instructions

### 1. Install Rust

Install the Rust toolchain via rustup:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone the Repository

```sh
git clone https://github.com/arrudagba/Rust-Web-Crawler
cd Rust-Web-Crawler
```

### 3. Build the Project

Option A: Debug Build

```sh
cargo build
```

Option B: Release Build

```sh
cargo build --release
```

### 4. Run the Crawler

Option A: Using `cargo run`

```sh
cargo run -- -d 2 -f urls.txt -e -v https://example.com
```

> Note: You must include `--` to separate cargo arguments from program arguments.

Option B: Using the compiled binary directly

- Debug build:

```sh
./target/debug/web_crawler -d 2 -f urls.txt -e -v https://example.com
```

- Release build:

```sh
./target/release/web_crawler -d 3 -fj crawl.json -v https://example.com
```

## Using Makefile (Optional)

This project includes a `Makefile` to simplify building and running the crawler.

Example Commands:

```sh
make build                        # Compile in debug mode
make release                      # Compile in release mode
make run ARGS="-d 2 -v https://example.com"
make run-bin ARGS="-d 1 -f output.txt https://example.com"
make run-release-bin ARGS="-d 3 -fj crawl.json -v https://example.com"
make clean                        # Clean build artifacts
sudo make install                 # Install binary to /usr/local/bin
```

Once installed, you can run the crawler from anywhere:

```sh
web_crawler -d 2 -v https://example.com
```

## Output

- **Text File Output** (`-f`): Writes visited URLs to a text file.
- **JSON File Output** (`-fj`): Writes visited URLs to a structured JSON file.
- **Request Error Logging** (`-e`): Logs failed requests (e.g., 404, timeout).
- **Verbose Output** (`-v`): Shows visited URLs in real time.

## Example Use Case

Crawl a site up to depth 3 and save all visited links in JSON format:

```sh
web_crawler -d 3 -fj crawl.json -v https://yourdomain.com
```

## Author

Developed by: [@arrudagba](https://github.com/arrudagba)
