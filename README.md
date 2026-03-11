# Atlas

An open-source, community-driven visual map of mathematics.

Atlas represents theorems, lemmas, definitions, and axioms as nodes in an interactive Directed Acyclic Graph (DAG). Dependencies between nodes form the edges. The system is entirely static — the frontend is plain HTML/JS and the backend is a Rust CLI that compiles Typst source files into a `graph.json` and pre-rendered SVG/PDF fragments.

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Content | Typst `.typ` files |
| Compiler + API | Rust (`axum`, `typst-syntax`, `reqwest`) |
| Graph viewer | `force-graph` JS library |
| IDE | Monaco Editor + `@myriaddreamin/typst.ts` (WASM) |
| Auth | GitHub OAuth 2.0 (popup flow, `repo` scope) |
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
```

For public/tunnel deployment, use `compiler/.env.public` with credentials for a separate "Atlas" OAuth App.

Submissions tagged `// tags: [demo]` skip the Git/PR step entirely (useful for testing).

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
