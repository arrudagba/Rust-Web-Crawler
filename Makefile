# Makefile for Rust-Web-Crawler

APP_NAME=web_crawler
INSTALL_PATH=/usr/local/bin

# Default target
all: build

# Build in debug mode
build:
	cargo build

# Build in release mode
release:
	cargo build --release

# Run in debug mode (pass args with ARGS="...")
run:
	cargo run -- $(ARGS)

# Run using the compiled debug binary
run-bin:
	./target/debug/$(APP_NAME) $(ARGS)

# Run using the compiled release binary
run-release-bin:
	./target/release/$(APP_NAME) $(ARGS)

# Clean build artifacts
clean:
	cargo clean

# Install the release binary to /usr/local/bin (may require sudo)
install: release
	cp ./target/release/$(APP_NAME) $(INSTALL_PATH)
	@echo "Installed to $(INSTALL_PATH)/$(APP_NAME)"
