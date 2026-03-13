# Atlas

An open-source, community-driven visual map of mathematics.

Atlas represents theorems, lemmas, definitions, and axioms as nodes in an interactive Directed Acyclic Graph (DAG). Dependencies between nodes form the edges. The system is entirely static — the frontend is plain HTML/JS and the backend is a Rust CLI that compiles Typst source files into a `graph.json` and pre-rendered SVG/PDF fragments.

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Content | Typst `.typ` files |
| Compiler + API | Rust (`axum`, `typst-syntax`, `reqwest`, `rusqlite`) |
| Graph viewer | `force-graph` JS library |
| IDE | Monaco Editor + `@myriaddreamin/typst.ts` (WASM) |
| Auth & Profiles | GitHub OAuth 2.0 (popup flow) and local SQLite (`atlas.db`) |
| Styling | Vanilla CSS (Dracula dark theme) |

---

## Documentation

Full documentation is on the [GitHub Wiki](../../wiki):

- **[Architecture](../../wiki/Architecture)** — system design and data flow
- **[Contributing](../../wiki/Contributing)** — submission format and body macros
- **[API Reference](../../wiki/API-Reference)** — REST endpoints
- **[Deployment](../../wiki/Deployment)** — local setup, public tunnel, and production

---

## Quick Start

```bash
# 1. Compile the math graph
make compile

# 2. Start the API server (local)
make server

# 3. Serve the frontend
make serve
```

See [Deployment](../../wiki/Deployment) for environment variable setup and the public tunnel workflow.

## Production domains

Atlas production is set up as:

- **Frontend (GitHub Pages):** `https://atlasmath.org`
- **Backend API:** `https://api.atlasmath.org`

`public/CNAME` is configured for `atlasmath.org`, and frontend API calls are configured in `public/js/config.js` via:

```js
window.ATLAS_API_URL = 'https://api.atlasmath.org';
```

For GitHub OAuth in production, set callback URL to:

- `https://api.atlasmath.org/api/auth/callback`

For webhook delivery in production, use:

- `https://api.atlasmath.org/api/github/webhook`

### Make targets

| Target | Description |
|--------|-------------|
| `make build` | Debug build |
| `make build-release` | Optimised release build |
| `make test` | Run integration tests |
| `make compile` | Compile `.typ` files → `graph.json` + SVG/PDF |
| `make server` | API server on `127.0.0.1:3000` |
| `make server-public` | API server on `0.0.0.0:3000` (tunnel/LAN) |
| `make serve` | Frontend on `localhost:8000` |
| `make tunnel` | ngrok tunnel with persistent static domain |
| `make full` | First-time setup: build → compile → server |

---

## Environment

Copy `compiler/.env.example` to `compiler/.env` and fill in your values. The server picks it up automatically on startup.

```bash
# GitHub OAuth App — "Atlas Dev" (local)
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...

# Fallback: server-side token used when no user OAuth token is present
GITHUB_TOKEN=ghp_...

# Webhook signing secret for GitHub events (pull_request, pull_request_review)
GITHUB_WEBHOOK_SECRET=...

# Optional: allow local replay of webhook payloads via /api/dev/replay-webhook
ATLAS_ENABLE_DEV_WEBHOOK_REPLAY=false
```

For public/tunnel deployment, use `compiler/.env.public` with credentials for a separate "Atlas" OAuth App.

Submissions tagged `// tags: [demo]` skip the Git/PR step entirely (useful for testing).

### Profile metrics & contribution graph

- `commits` increase **only when a PR is merged into `main`** (via GitHub webhook `pull_request` event).
- `reviews` increase on each `pull_request_review` `submitted` event, regardless of whether the PR is merged or closed later.
- The profile page includes a GitHub-style contribution heatmap generated from daily contribution buckets stored in `atlas.db`.

---

## Pages

| Page | File | Purpose |
|------|------|---------|
| Graph viewer | `index.html` | Browse the math graph |
| Contributor portal | `submit.html` | Upload a `.typ` file |
| IDE | `ide.html` | Write, preview, and publish Typst in-browser |

---

## License

MIT
