# KnowledgeGATEunlocker task runner

default:
    @just --list

# Build all crates
build:
    cargo build

# Build release
release:
    cargo build --release

# Run CLI
run *args:
    cargo run -p kgate -- {{args}}

# Run voice exam
voice *args:
    cargo run -p kgate -- voice {{args}}

# Watch and rebuild on changes
watch:
    cargo watch -x 'build'

# Watch and run on changes
dev:
    cargo watch -x 'run -p kgate'

# Run tests
test:
    cargo test

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without changing
fmt-check:
    cargo fmt -- --check

# Initialize database
db-init:
    sqlx database create
    sqlx migrate run

# Create new migration
db-migrate name:
    sqlx migrate add {{name}}

# Reset database
db-reset:
    sqlx database drop -y
    sqlx database create
    sqlx migrate run

# Full check (fmt, lint, test)
check: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Install to ~/.local/bin
install: release
    cp target/release/kgate ~/.local/bin/kgate
    @echo "✓ Installed kgate to ~/.local/bin"

# Uninstall
uninstall:
    rm -f ~/.local/bin/kgate
    @echo "✓ Removed kgate"
