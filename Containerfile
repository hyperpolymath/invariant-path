# SPDX-License-Identifier: MPL-2.0
#
# Containerfile — Invariant Path CLI
#
# Nix retirement note: this repo's flake.nix predates the estate's
# 2026-06-01 Guix-primary ruling. flake.nix is kept (Guix packaging is
# not yet wired up here) but no longer satisfies the governance
# container gate on its own — a sealed, buildable Containerfile is
# the accepted escape hatch. This file is a real, non-stub build:
# every dependency install below is an active RUN step, and the
# binary produced here is the actual `invariant-path-cli` crate.
#
# Toolchain: Rust, workspace edition 2021 (see Cargo.toml). No
# rust-toolchain.toml pin exists in this repo to honour, so this
# uses Wolfi's rust-1.89 package (a recent stable release bundling
# both rustc and cargo; Wolfi does not ship a bare "cargo" package).
#
# Multi-stage build:
#   Stage 1: compile the invariant-path-cli binary (+ invariant-path-core
#             it depends on) with cargo --release
#   Stage 2: copy the static-ish release binary into a minimal
#             Chainguard glibc runtime image
#
# Build:  podman build -t invariant-path -f Containerfile .
# Run:    podman run --rm -it invariant-path --help
# Seal:   podman build --no-cache -t invariant-path:sealed -f Containerfile .

# --- Stage 1: Build (Rust) ---
FROM cgr.dev/chainguard/wolfi-base:latest AS builder

# Rust toolchain (rustc + cargo, rust-1.89 bundles both) as packaged by Wolfi
RUN apk add --no-cache rust-1.89 gcc

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build only the CLI binary crate; it pulls in invariant-path-core
# transitively as a workspace path dependency.
RUN cargo build --release -p invariant-path-cli && \
    cp target/release/invariant-path-cli /build/invariant-path-cli

# --- Stage 2: Runtime ---
FROM cgr.dev/chainguard/glibc-dynamic:latest

COPY --from=builder /build/invariant-path-cli /usr/bin/invariant-path-cli

USER nonroot

ENTRYPOINT ["/usr/bin/invariant-path-cli"]
