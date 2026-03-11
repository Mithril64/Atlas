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
# Required for PR creation when no user OAuth token is provided
export GITHUB_TOKEN=ghp_...

# Required for GitHub OAuth login (register an OAuth App on github.com/settings/developers)
export GITHUB_CLIENT_ID=your_client_id
export GITHUB_CLIENT_SECRET=your_client_secret
# Callback URL must match what's registered in your OAuth App:
export GITHUB_REDIRECT_URL=http://127.0.0.1:3000/api/auth/callback
```

### Running Tests

```bash
cd compiler && cargo test
```

20 unit tests covering `ingest_submission` (happy paths, edge cases, error conditions). No external services required.


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
