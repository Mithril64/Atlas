# Atlas Deployment

## Local Development

### Prerequisites

- **Rust** (stable): [rustup.rs](https://rustup.rs/)
- **Typst CLI** in `$PATH`: [github.com/typst/typst/releases](https://github.com/typst/typst/releases)
- **Git** configured with push access to the repository

### Build & run

```bash
# Compile the graph from all .typ files
cd compiler && cargo run --release

# Start the submission API server (separate terminal)
cd compiler && cargo run --release -- server
# → http://127.0.0.1:3000
```

Then open `public/index.html` in a browser (any static server or directly from disk).

### Environment variables

```bash
export GITHUB_TOKEN=ghp_...   # Required for PR creation on submission
```

---

## Production

The server is a single Axum binary. A minimal systemd unit:

```ini
[Unit]
Description=Atlas API Server
After=network.target

[Service]
User=atlas
WorkingDirectory=/home/atlas/Atlas/compiler
ExecStart=/home/atlas/Atlas/compiler/target/release/atlas-compiler server
Environment=GITHUB_TOKEN=ghp_...
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Serve the `public/` directory with Nginx or any CDN. Proxy `/api/` to `127.0.0.1:3000`:

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:3000;
}

location / {
    root /home/atlas/Atlas/public;
    try_files $uri /index.html;
}
```

### After new submissions are merged

Re-run the compiler to regenerate the graph and SVG/PDF fragments:

```bash
cd compiler && cargo run --release
```

There is no auto-recompilation watcher currently — this is a manual step.
