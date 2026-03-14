#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${REPO_DIR:-/root/Atlas}"

export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
	if [ -f "$HOME/.cargo/env" ]; then
		# shellcheck disable=SC1090
		. "$HOME/.cargo/env"
	fi
fi

if ! command -v cargo >/dev/null 2>&1; then
	echo "[deploy] ERROR: cargo not found for user '$USER'. Install Rust via rustup for this user."
	exit 127
fi

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
