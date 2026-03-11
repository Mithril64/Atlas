# Atlas

An open-source, community-driven visual map of mathematics.

Atlas represents theorems, lemmas, definitions, and axioms as nodes in an interactive Directed Acyclic Graph (DAG). Dependencies between nodes form the edges. The system is entirely static — the frontend is plain HTML/JS and the backend is a Rust CLI that compiles Typst source files into a `graph.json` and pre-rendered SVG/PDF fragments.

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Content | Typst `.typ` files |
| Compiler | Rust (`typst-syntax` crate for AST parsing) |
| Submission API | Axum HTTP server (local, `127.0.0.1:3000`) |
| Graph viewer | `force-graph` JS library + HTMX |
| IDE | Monaco Editor + `@myriaddreamin/typst.ts` (WASM) |
| Styling | Vanilla CSS (Dracula-inspired dark theme) |

---

## Documentation

Full documentation is on the [GitHub Wiki](../../wiki):

- **[Architecture](../../wiki/Architecture)** — system design and data flow
- **[Contributing](../../wiki/Contributing)** — submission format and body macros
- **[API Reference](../../wiki/API-Reference)** — REST endpoints
- **[Deployment](../../wiki/Deployment)** — local setup and production install

---

## Running Locally

### Prerequisites
- [Rust](https://rustup.rs/) (stable)
- [Typst CLI](https://github.com/typst/typst/releases) (`typst` in PATH)
- Git (configured with push access for submissions to work)

### 1. Compile the graph

```bash
cd compiler
cargo run --release
```

Reads all `.typ` files in `../math/`, writes `../public/json/graph.json` and `../public/nodes/*.svg/.pdf`.

### 2. Start the API server

```bash
cd compiler
cargo run --release -- server
```

Starts on `http://127.0.0.1:3000`. Required for the submit portal and IDE publish button.

### 3. Serve the frontend

Open `public/index.html` directly in a browser, or serve the `public/` directory with any static file server.

---

## Environment

The submission server requires a `GITHUB_TOKEN` env variable with repo write access to open pull requests on submission:

```bash
export GITHUB_TOKEN=ghp_...
cargo run --release -- server
```

Submissions tagged `// tags: [demo]` skip the Git/PR step and just return success (useful for local testing).

---

## Pages

| Page | URL | Purpose |
|------|-----|---------|
| Graph viewer | `index.html` | Browse the math graph |
| Contributor portal | `submit.html` | Upload a `.typ` file |
| IDE | `ide.html` | Write and preview Typst in-browser |

---

## License

MIT
