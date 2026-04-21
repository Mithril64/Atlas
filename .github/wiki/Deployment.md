# Deployment

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Typst](https://typst.app/) CLI (`typst compile` must be on `$PATH`)
- `make`
- Python 3 (for `make serve` — uses `python3 -m http.server`)

---

## Local Development

**1. Create `compiler/.env`:**

```bash
GITHUB_CLIENT_ID=your_local_oauth_app_client_id
GITHUB_CLIENT_SECRET=your_local_oauth_app_client_secret
GITHUB_TOKEN=ghp_your_personal_access_token
GITHUB_WEBHOOK_SECRET=any_secret_string
ATLAS_ENABLE_DEV_WEBHOOK_REPLAY=false
```

Create a GitHub OAuth App at [github.com/settings/developers](https://github.com/settings/developers) with callback URL `http://127.0.0.1:3000/api/auth/callback`.

**2. Run the stack:**

```bash
make compile      # parse math/ → graph.json + SVG/PDF
make server       # API on 127.0.0.1:3000
make serve        # frontend on localhost:8000 (separate terminal)
```

Or for a combined watch loop: `make dev` (recompiles on `math/` changes, serves frontend).

**Skipping GitHub credentials:** tag your test submissions `// tags: [demo]` — the server accepts them without attempting any Git or GitHub API calls.

---

## Public Tunnel (ngrok)

Useful for testing OAuth and webhooks without a VPS.

**1. Set up ngrok:**
- Create a free account at [ngrok.com](https://ngrok.com)
- `ngrok config add-authtoken <your-token>`
- Note your free static domain (e.g. `your-name.ngrok-free.app`)

**2. Create `compiler/.env.public`** with the same fields as `.env`, plus:

```bash
NGROK_DOMAIN=your-name.ngrok-free.app
GITHUB_REDIRECT_URL=https://your-name.ngrok-free.app/api/auth/callback
```

Create a second GitHub OAuth App with callback URL `https://your-name.ngrok-free.app/api/auth/callback`.

**3. Run:**

```bash
make server-public   # API on 0.0.0.0:3000 with .env.public
make serve-public    # frontend on 0.0.0.0:8000 with API URL injected
make tunnel          # ngrok tunnel → your-name.ngrok-free.app
```

For a throwaway tunnel without an account: `make tunnel-quick` (uses cloudflared; URL changes every restart).

---

## Production (VPS)

The production stack runs the Axum server behind a reverse proxy (Cloudflare/Nginx) and serves the frontend via GitHub Pages.

**Frontend (GitHub Pages):**  
Served from the `public/` directory. `public/CNAME` contains `atlasmath.org`. `make serve` is not used in production — GitHub Pages serves the static files directly.

`public/js/config.js` must contain:
```js
window.ATLAS_API_URL = 'https://api.atlasmath.org';
```

**Backend (VPS):**  
Run with `make server-prod`, which binds to `127.0.0.1:3000` and loads `compiler/.env.public`. The reverse proxy forwards `api.atlasmath.org` to port 3000.

### Environment

On the VPS, set up `compiler/.env.public`:

```bash
GITHUB_CLIENT_ID=...          # production OAuth App credentials
GITHUB_CLIENT_SECRET=...
GITHUB_TOKEN=ghp_...
GITHUB_REDIRECT_URL=https://api.atlasmath.org/api/auth/callback
GITHUB_WEBHOOK_SECRET=...
ATLAS_LINK_BASE=https://atlasmath.org
```

### CI/CD

| Workflow | Trigger | Action |
|----------|---------|--------|
| `deploy-pages.yml` | Push to `main` | Deploys `public/` to GitHub Pages |
| `deploy-backend.yml` | Push to `main` (compiler or math changes) | SSH into VPS, runs `scripts/deploy_backend.sh` |
| `cleanup-branches.yml` | PR closed or merged | Deletes the `submission-*` branch |
| `sync-wiki.yml` | Push to `main` (`.github/wiki/` changes) | Syncs `.github/wiki/` to the GitHub wiki |

### GitHub Webhook

In the repository settings, add a webhook:

- **Payload URL:** `https://api.atlasmath.org/api/github/webhook`
- **Content type:** `application/json`
- **Secret:** must match `GITHUB_WEBHOOK_SECRET` in `.env.public`
- **Events:** `Pull requests`, `Pull request reviews`

### Replaying webhooks locally

Set `ATLAS_ENABLE_DEV_WEBHOOK_REPLAY=true` in `.env`, then POST a GitHub webhook payload to `http://127.0.0.1:3000/api/dev/replay-webhook` with the `X-GitHub-Event` header set to `pull_request` or `pull_request_review`.
