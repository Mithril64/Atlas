.PHONY: build test compile server server-public server-prod serve serve-public dev tunnel tunnel-quick clean

# ─── Build & test ─────────────────────────────────────────────────────────────

## Type-check / debug build
build:
	cd compiler && cargo build

## Run all tests
test:
	cd compiler && cargo test
	@echo "✓ All tests passed"

# ─── Compile graph ────────────────────────────────────────────────────────────

## Parse math/ → public/json/graph.json + public/nodes/{id}.svg/.pdf
compile:
	cd compiler && DOTENV_FILE=$${DOTENV_FILE:-.env} cargo run --release
	@echo "✓ Graph compiled"

# ─── Servers ──────────────────────────────────────────────────────────────────

## Local API on 127.0.0.1:3000 (loads .env)
server:
	cd compiler && cargo run --release -- server

## LAN / tunnel API on 0.0.0.0:3000 (loads .env.public)
server-public:
	cd compiler && DOTENV_FILE=.env.public SERVER_HOST=0.0.0.0 SERVER_PORT=3000 cargo run --release -- server

## Production API on 127.0.0.1:3000 behind reverse proxy (loads .env.public)
server-prod:
	cd compiler && DOTENV_FILE=.env.public SERVER_HOST=127.0.0.1 SERVER_PORT=3000 cargo run --release -- server

# ─── Frontend ─────────────────────────────────────────────────────────────────

## Serve frontend on localhost:8000 pointed at local API
serve:
	@echo "window.ATLAS_API_URL = 'http://127.0.0.1:3000';" > public/js/config.js
	cd public && python3 -m http.server 8000

## Serve frontend on 0.0.0.0:8000 pointed at ngrok domain (reads .env.public)
serve-public:
	@DOMAIN=$$(grep NGROK_DOMAIN compiler/.env.public 2>/dev/null | cut -d= -f2 | tr -d '\r'); \
	if [ -z "$$DOMAIN" ]; then \
		echo "Error: set NGROK_DOMAIN in compiler/.env.public"; exit 1; \
	fi; \
	echo "window.ATLAS_API_URL = 'https://$$DOMAIN';" > public/js/config.js
	cd public && python3 -m http.server 8000 --bind 0.0.0.0

# ─── Dev (combined) ───────────────────────────────────────────────────────────

## Compile, then watch math/ for changes and serve the frontend
## Requires cargo-watch: cargo install cargo-watch
dev:
	$(MAKE) compile
	cd compiler && cargo watch -w ../math -x run & $(MAKE) serve

# ─── Tunnel ───────────────────────────────────────────────────────────────────

## ngrok tunnel with persistent static domain (set NGROK_DOMAIN in .env.public)
## Usage: make server-public (t1) + make serve-public (t2) + make tunnel (t3)
tunnel:
	@DOMAIN=$$(grep NGROK_DOMAIN compiler/.env.public 2>/dev/null | cut -d= -f2); \
	if [ -z "$$DOMAIN" ]; then \
		echo "Error: set NGROK_DOMAIN=your-name.ngrok-free.app in compiler/.env.public"; exit 1; \
	fi; \
	echo "→ API:      https://$$DOMAIN"; \
	echo "→ Frontend: http://localhost:8000"; \
	ngrok http --domain=$$DOMAIN 3000

## Throwaway cloudflared tunnel — no account needed, URL changes on restart
tunnel-quick:
	cloudflared tunnel --url http://localhost:3000 &
	@sleep 2
	cloudflared tunnel --url http://localhost:8000

# ─── Housekeeping ─────────────────────────────────────────────────────────────

clean:
	cd compiler && cargo clean
	@echo "✓ Build artefacts removed"
