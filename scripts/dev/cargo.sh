#!/bin/sh
# Run cargo inside a Linux container (dav1d is not available on Windows).
# Build the image once: docker build -t sonm-dev scripts/dev
# Usage: sh scripts/dev/cargo.sh check --workspace
ROOT="$(cd "$(dirname "$0")/../.." && { pwd -W 2>/dev/null || pwd; })"
MSYS_NO_PATHCONV=1 docker run --rm \
  -v "$ROOT:/src" \
  -v "$HOME/.cargo/registry:/usr/local/cargo/registry" \
  -v "$HOME/.cargo/git:/usr/local/cargo/git" \
  -v sonm-target:/target -e CARGO_TARGET_DIR=/target -w /src \
  sonm-dev cargo "$@"
