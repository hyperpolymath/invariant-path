import? "contractile.just"

# Invariant Path - Justfile
# Build automation and development tasks

# Default target
default:
    just --list

# Build the project
build:
    cargo build --release

# Run the CLI
run-cli:
    cargo run --release --bin invariant-path-cli

# Ground the AffineScript "different faces, same cube" invariant (faces profile)
same-cube corpus="examples/same-cube/greet":
    ./scripts/verify-same-cube.sh {{corpus}} --out .machine_readable/audits/same-cube.jsonl

# Machine-check the same-cube proofs (Agda, --safe)
proofs:
    agda --safe proofs/SameCube.agda

# Run the TUI
run-tui:
    ./invariant-path-launcher --auto

# Build and run
build-run:
    just build
    just run-tui

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Lint code
lint:
    cargo clippy

# Clean build artifacts
clean:
    cargo clean
    rm -f /tmp/invariant-path-last-scan.json /tmp/invariant-path-launcher.log

# Install desktop integration
install:
    cp invariant-path-launcher /var/mnt/eclipse/repos/.desktop-tools/invariant-path-launcher.sh
    chmod +x /var/mnt/eclipse/repos/.desktop-tools/invariant-path-launcher.sh

# Scan current directory
scan:
    ./invariant-path-launcher --scan . generic

# Open last scan output
open-output:
    ./invariant-path-launcher --open-output

# Show status
status:
    ./invariant-path-launcher --status

# Start server
start:
    ./invariant-path-launcher --start

# Stop server
stop:
    ./invariant-path-launcher --stop

# Web UI (experimental)
web:
    ./invariant-path-launcher --web

# Cross-launch NQC
nqc:
    /var/mnt/eclipse/repos/nextgen-databases/nqc/nqc-launcher.sh --auto

# Show help
help:
    just --list