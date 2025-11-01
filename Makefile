.PHONY: check fmt clippy test build clean precommit help

help:
	@echo "Available targets:"
	@echo "  make precommit  - Run all quality checks (fmt, clippy, build)"
	@echo "  make fmt        - Format code with cargo fmt"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make check      - Run cargo check"
	@echo "  make build      - Build the project"
	@echo "  make test       - Run tests"
	@echo "  make clean      - Clean build artifacts"

# Format code
fmt:
	cargo fmt

# Run clippy with strict warnings
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Quick check without building
check:
	cargo check

# Build release version
build:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Run all pre-commit checks
precommit: fmt clippy build
	@echo "✓ All checks passed!"
