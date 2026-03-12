# Atlas Deployment

## Local Development

### Prerequisites

- **Rust** (stable): [rustup.rs](https://rustup.rs/)
- **Typst CLI** in `$PATH`: [github.com/typst/typst/releases](https://github.com/typst/typst/releases)
- **Git** configured with push access to the repository

### Build & run

```bash
make compile   # compile all .typ files → graph.json + SVG/PDF
make server    # start the API on http://127.0.0.1:3000
make serve     # serve public/ on http://localhost:8000  (separate terminal)
```

Or for a first-time full setup:

```bash
make full      # build → compile → server (also runs quickstart.sh full)
```

### Environment variables

Create `compiler/.env` from the example:

```bash
cp compiler/.env.example compiler/.env
# then edit with your values
```

```bash
# GitHub OAuth App — "Atlas Dev"
# Register at https://github.com/settings/developers
# Callback URL: http://127.0.0.1:3000/api/auth/callback
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret

# Fallback server-side token (used when no user OAuth token is provided)
# Required scope: repo
# GITHUB_TOKEN=ghp_...

# Webhook signing secret (GitHub Webhooks -> Secret)
# GITHUB_WEBHOOK_SECRET=...

# Optional local helper for replaying webhook payloads
# ATLAS_ENABLE_DEV_WEBHOOK_REPLAY=false
```

### Webhook setup (required for accurate profile metrics)

Configure a GitHub webhook for your repository:

- **Payload URL**: `http://127.0.0.1:3000/api/github/webhook` (or your public URL in tunnel/prod)
- **Content type**: `application/json`
- **Secret**: same value as `GITHUB_WEBHOOK_SECRET`
- **Events**:
    - Pull requests
    - Pull request reviews

This drives profile counters and heatmap data:

- commits count only when PRs are merged into `main`
- reviews count on `pull_request_review.submitted`

### Local webhook replay (optional)

For local testing without GitHub delivery:

- set `ATLAS_ENABLE_DEV_WEBHOOK_REPLAY=true`
- POST a webhook-shaped payload to `/api/dev/replay-webhook`
- include `X-GitHub-Event` header (`pull_request` or `pull_request_review`)

The server auto-loads `.env` on startup via `dotenv`. No need to `export` manually.

### Running Tests

```bash
make test
# or: cd compiler && cargo test
```

30 integration tests across 4 files in `compiler/tests/`:

| File | Coverage |
|------|----------|
| `ingest_submission.rs` | Node type variants |
| `deps_and_tags.rs` | Dep/tag parsing, quoting, normalisation |
| `body_and_output.rs` | Body extraction, output structure |
| `errors.rs` | Missing fields, invalid input |

---

## Public Tunnel (for sharing with testers)

Uses **ngrok** with a free static domain (URL never changes).

### One-time setup

```bash
# 1. Install ngrok & authenticate
ngrok config add-authtoken <your-token>

# 2. Add to compiler/.env.public
NGROK_DOMAIN=your-name.ngrok-free.dev
GITHUB_CLIENT_ID=<Atlas OAuth App client ID>
GITHUB_CLIENT_SECRET=<Atlas OAuth App client secret>
GITHUB_REDIRECT_URL=https://your-name.ngrok-free.dev/api/auth/callback

# 3. Set ATLAS_API_URL in public/js/config.js
window.ATLAS_API_URL = 'https://your-name.ngrok-free.dev';

# 4. Update GitHub "Atlas" OAuth App callback URL to the ngrok domain
```

You need **two separate GitHub OAuth Apps**:
- **Atlas Dev** — callback `http://127.0.0.1:3000/api/auth/callback` → credentials in `.env`
- **Atlas** — callback `https://your-name.ngrok-free.dev/api/auth/callback` → credentials in `.env.public`

### Running publicly

```bash
# Terminal 1
make server-public   # API on 0.0.0.0:3000, loads .env.public automatically

# Terminal 2
make serve-public    # frontend on 0.0.0.0:8000

# Terminal 3
make tunnel          # starts ngrok with your persistent static domain
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
ExecStart=/home/atlas/Atlas/compiler/target/release/math-graph-compiler server
EnvironmentFile=/home/atlas/Atlas/compiler/.env
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Serve the `public/` directory with Nginx. Proxy `/api/` to `127.0.0.1:3000`:

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:3000;
}

location / {
    root /home/atlas/Atlas/public;
    try_files $uri $uri/ /index.html;
}
```

### Regenerating the graph after merged submissions

```bash
make compile
```

Submission branches are cleaned up automatically by the GitHub Actions workflow in `.github/workflows/cleanup-branches.yml` — any branch named `submission-*` is deleted when its PR is merged or closed.
