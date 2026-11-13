.PHONY: all build test clippy fmt fmt-check clean check screenshot gui release bench update coverage help

# Default target
all: build

# Build the workspace
build:
	cargo build

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
clippy:
	cargo clippy --workspace --all-targets -- -D warnings

# Alias used by the pnpm/turbo shim (crates/package.json)
lint: clippy

# Format code
fmt:
	cargo fmt --all

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Clean build artifacts
clean:
	cargo clean

# Run all checks (test + clippy + fmt)
check: test clippy fmt-check

# Run CLI against a fixture page (headless screenshot)
screenshot:
	cargo run -p browsers-cli -- screenshot \
		"file://$(CURDIR)/crates/browsers-cli/tests/fixtures/rendering/mvp.html" \
		--output /tmp/screenshot.png

# Open a headed window (loads https://example.com)
gui:
	cargo run -p browsers-cli -- open

# Build release binaries
release:
	cargo build --release

# Run benchmarks
bench:
	cargo test --release --workspace

# Update dependencies
update:
	cargo update

# Per-crate test targets (workspace members)
test-core:
	cargo test -p browsers-core

test-headless:
	cargo test -p browsers-headless

test-cli:
	cargo test -p browsers-cli

# Run integration tests only
test-integration:
	cargo test -p browsers-cli --test '*'

# Help
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build the workspace (default)"
	@echo "  build        - Build the workspace"
	@echo "  test         - Run all tests"
	@echo "  clippy       - Run clippy lints"
	@echo "  fmt          - Format code"
	@echo "  fmt-check    - Check formatting"
	@echo "  clean        - Clean build artifacts"
	@echo "  check        - Run all checks (test + clippy + fmt)"
	@echo "  screenshot   - Screenshot the MVP fixture page"
	@echo "  gui          - Open the MVP fixture page in headed mode"
	@echo "  release      - Build release binaries"
	@echo "  bench        - Run benchmarks"
	@echo "  update       - Update dependencies"
	@echo "  test-core    - Test browsers-core"
	@echo "  test-headless - Test browsers-headless"
	@echo "  test-cli     - Test browsers-cli"
	@echo "  test-integration - Run CLI integration tests"
	@echo "  help         - Show this help"
