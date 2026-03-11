.PHONY: build build-release test compile server serve watch dev tunnel tunnel-full full clean

# ─── Build ──────────────────────────────────────────────────────────────────

## Debug build (fast iteration)
build:
	cd compiler && cargo build

## Optimised release build
build-release:
	cd compiler && cargo build --release
	@echo "✓ Release binary: compiler/target/release/math-graph-compiler"

# ─── Test ───────────────────────────────────────────────────────────────────

## Run the full integration test suite (tests live in compiler/tests/)
test:
	cd compiler && cargo test
	@echo "✓ All tests passed"

# ─── Compile graph ──────────────────────────────────────────────────────────

## Compile all .typ files → public/json/graph.json + public/nodes/{id}.svg/.pdf
compile:
	cd compiler && cargo run --release
	@echo "✓ Graph compiled"

# ─── Local servers ──────────────────────────────────────────────────────────

## Start the Axum API on localhost:3000 (default, safe)
server:
	cd compiler && cargo run --release -- server

## Start the Axum API on 0.0.0.0:3000 (LAN / hosting — pairs with `make tunnel`)
server-public:
	SERVER_HOST=0.0.0.0 cd compiler && cargo run --release -- server

## Serve the static frontend on port 8000
serve:
	cd public && python3 -m http.server 8000

## Serve the static frontend on all interfaces (for LAN / tunnel users)
serve-public:
	cd public && python3 -m http.server 8000 --bind 0.0.0.0

## Watch math/ for changes and auto-recompile the graph
watch:
	cd compiler && cargo watch -w ../math -x run

# ─── Public tunnel ──────────────────────────────────────────────────────────
#
# Persistent URL (recommended) — uses ngrok free static domain:
#   1. Sign up free at https://ngrok.com
#   2. ngrok config add-authtoken <your-token>
#   3. Add NGROK_DOMAIN=your-name.ngrok-free.app to compiler/.env.public
#   4. Set GitHub OAuth callback once to https://your-name.ngrok-free.app/api/auth/callback
#
# Usage:
#   Terminal 1: make server-public
#   Terminal 2: make serve-public
#   Terminal 3: make tunnel

tunnel:
	@DOMAIN=$$(grep NGROK_DOMAIN compiler/.env.public 2>/dev/null | cut -d= -f2); \
	if [ -z "$$DOMAIN" ]; then \
		echo "Error: set NGROK_DOMAIN=your-name.ngrok-free.app in compiler/.env.public"; exit 1; \
	fi; \
	echo "→ API tunnel:     https://$$DOMAIN"; \
	echo "→ Frontend:       http://localhost:8000 (share via a separate frontend tunnel)"; \
	ngrok http --domain=$$DOMAIN 3000

## Quick throwaway tunnel (no account needed, URL changes every restart)
tunnel-quick:
	@echo "→ Starting two temporary cloudflared tunnels (URL will change on restart)"
	cloudflared tunnel --url http://localhost:3000 &
	@sleep 2
	cloudflared tunnel --url http://localhost:8000

# ─── Combined dev / full ───────────────────────────────────────────────────

## Full dev cycle: compile graph, then start watch + local serve
dev:
	@echo "Starting Atlas Development Suite..."
	$(MAKE) compile
	$(MAKE) watch & $(MAKE) serve

## First-time setup: build, compile, and start the server
full:
	bash quickstart.sh full

# ─── Housekeeping ───────────────────────────────────────────────────────────

clean:
	cd compiler && cargo clean
	@echo "✓ Build artefacts removed"
