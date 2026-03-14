#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${REPO_DIR:-/root/Atlas}"

cd "$REPO_DIR"

echo "[deploy] Fetching latest main..."
git fetch origin
git reset --hard origin/main

echo "[deploy] Building backend release binary..."
cd compiler
cargo build --release

echo "[deploy] Restarting atlas-api service..."
sudo systemctl restart atlas-api
sudo systemctl --no-pager --full status atlas-api

echo "[deploy] Done."
