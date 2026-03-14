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
echo "[deploy] Deployed commit: $(git rev-parse --short HEAD)"

echo "[deploy] Debug: PATH is $PATH"
echo "[deploy] Debug: using cargo at ${CARGO_BIN}"
if command -v typst >/dev/null 2>&1; then
	echo "[deploy] Debug: typst version -> $(typst --version)"
else
	echo "[deploy] WARNING: typst not found in PATH during deploy (compiles will fail)"
fi

echo "[deploy] Debug: env files in compiler/ (listing only)"
ls -l "$REPO_DIR/compiler" | sed -n '1,12p'
if [ -f "$REPO_DIR/compiler/.env.public" ]; then
	echo "[deploy] Debug: compiler/.env.public exists (size $(stat -c%s "$REPO_DIR/compiler/.env.public") bytes)"
elif [ -f "$REPO_DIR/compiler/.env" ]; then
	echo "[deploy] Debug: compiler/.env exists (size $(stat -c%s "$REPO_DIR/compiler/.env") bytes)"
else
	echo "[deploy] WARNING: no .env or .env.public found in compiler/"
fi

echo "[deploy] Building backend release binary via Makefile..."
make build-release

echo "[deploy] Debug: newest backend artifacts after build-release:"
find "$REPO_DIR/compiler/target/release" -maxdepth 1 -type f -printf '%TY-%Tm-%Td %TH:%TM:%TS %f\n' | sort | tail -n 5

echo "[deploy] Debug: ensuring nodes directory exists"
mkdir -p "$REPO_DIR/public/nodes"

echo "[deploy] Compiling graph assets via Makefile (graph.json + nodes/*.svg/*.pdf)..."
DOTENV_FILE=.env.public make compile

echo "[deploy] Debug: compile finished; summarizing outputs"
if [ -f "$REPO_DIR/public/json/graph.json" ]; then
	echo "[deploy] Debug: graph.json size $(stat -c%s "$REPO_DIR/public/json/graph.json") bytes"
else
	echo "[deploy] WARNING: graph.json missing after compile"
fi
SVG_COUNT_POST=$(find "$REPO_DIR/public/nodes" -maxdepth 1 -type f -name '*.svg' | wc -l | tr -d ' ')
PDF_COUNT_POST=$(find "$REPO_DIR/public/nodes" -maxdepth 1 -type f -name '*.pdf' | wc -l | tr -d ' ')
echo "[deploy] Debug: node artifact counts => svg=$SVG_COUNT_POST pdf=$PDF_COUNT_POST"

echo "[deploy] Setting frontend API base URL for production..."
cat > "$REPO_DIR/public/js/config.js" <<'EOF'
window.ATLAS_API_URL = 'https://api.atlasmath.org';
EOF

echo "[deploy] Validating compiled frontend artifacts..."
if [ ! -s "$REPO_DIR/public/json/graph.json" ]; then
	echo "[deploy] ERROR: public/json/graph.json missing or empty after compile"
	exit 1
fi

if ! find "$REPO_DIR/public/nodes" -maxdepth 1 -type f \( -name '*.svg' -o -name '*.pdf' \) | grep -q .; then
	echo "[deploy] ERROR: no node SVG/PDF outputs found in public/nodes"
	exit 1
fi

EXPECTED_NODES=$(python3 - <<'PY'
import json
from pathlib import Path
p = Path("/root/Atlas/public/json/graph.json")
data = json.loads(p.read_text()) if p.exists() else []
print(len(data) if isinstance(data, list) else 0)
PY
)
SVG_COUNT=$(find "$REPO_DIR/public/nodes" -maxdepth 1 -type f -name '*.svg' | wc -l | tr -d ' ')

if [ "$SVG_COUNT" -lt "$EXPECTED_NODES" ]; then
  echo "[deploy] ERROR: node artifact mismatch (expected $EXPECTED_NODES SVGs from graph.json, found $SVG_COUNT)"
  exit 1
fi

echo "[deploy] Newest node artifacts:"
find "$REPO_DIR/public/nodes" -maxdepth 1 -type f \( -name '*.svg' -o -name '*.pdf' \) -printf '%TY-%Tm-%Td %TH:%TM:%TS %f\n' | sort | tail -n 5

echo "[deploy] Restarting atlas-api service..."
sudo systemctl restart atlas-api
sudo systemctl --no-pager --full status atlas-api

echo "[deploy] Debug: recent atlas-api journal entries"
sudo journalctl -u atlas-api -n 40 --no-pager || true

echo "[deploy] Done."
