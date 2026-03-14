#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${REPO_DIR:-/root/Atlas}"

# Determine likely home directory for rustup/cargo in non-login SSH sessions
DEPLOY_HOME="${HOME:-}"
if [ -z "$DEPLOY_HOME" ] || [ ! -d "$DEPLOY_HOME" ]; then
	DEPLOY_HOME="/root"
fi

export PATH="$DEPLOY_HOME/.cargo/bin:/root/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
	if [ -f "$DEPLOY_HOME/.cargo/env" ]; then
		# shellcheck disable=SC1090
		. "$DEPLOY_HOME/.cargo/env"
	elif [ -f "/root/.cargo/env" ]; then
		# shellcheck disable=SC1090
		. "/root/.cargo/env"
	fi
fi

if ! command -v cargo >/dev/null 2>&1; then
	if [ -x "/root/.cargo/bin/cargo" ]; then
		CARGO_BIN="/root/.cargo/bin/cargo"
	elif [ -x "$DEPLOY_HOME/.cargo/bin/cargo" ]; then
		CARGO_BIN="$DEPLOY_HOME/.cargo/bin/cargo"
	else
		echo "[deploy] ERROR: cargo not found for user '${USER:-unknown}'. Install Rust via rustup for this user."
		exit 127
	fi
else
	CARGO_BIN="$(command -v cargo)"
fi

cd "$REPO_DIR"

echo "[deploy] Fetching latest main..."
git fetch origin
git reset --hard origin/main

echo "[deploy] Building backend release binary..."
cd compiler
"$CARGO_BIN" build --release

echo "[deploy] Restarting atlas-api service..."
sudo systemctl restart atlas-api
sudo systemctl --no-pager --full status atlas-api

echo "[deploy] Done."
