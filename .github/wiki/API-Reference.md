# Atlas API Reference

Base URL (local): `http://127.0.0.1:3000`  
Base URL (public tunnel): `https://alva-keyed-unexplainably.ngrok-free.dev`  
Configured in `public/js/config.js` via `window.ATLAS_API_URL`.

---

## GET `/api/graph`

Returns the compiled graph as a JSON array.

**Response `200`:**
```json
[
  {
    "id": "thm-bolzano-weierstrass",
    "node_type": "theorem",
    "deps": ["def-bounded-sequence"],
    "tags": ["analysis"],
    "body": "#statement[...]..."
  }
]
```

**Response `200` (not yet compiled):**
```json
{"error": "No graph"}
```

---

## POST `/api/submit`

Submit a new `.typ` file.

**Content-Type:** `multipart/form-data`  
**Field:** `file` — a `.typ` file  
**Header (optional):** `Authorization: Bearer <github_oauth_token>` — if provided, the PR is opened on behalf of that user. Falls back to the server's `GITHUB_TOKEN` env var.

**Success `200`** (non-demo submission):
```json
{"status": "success", "pr_url": "https://github.com/Mithril64/Atlas/pull/42"}
```

**Success `200`** (demo submission — `tags: [demo]`, skips Git):
```json
{"status": "success", "id": "thm-my-theorem"}
```

**Error `400`** — validation failure (plain text):
```
No id
No type
```

**Error `500`:**
```
Git push failed
GitHub API error 404: Not Found
No GitHub token — log in with GitHub to submit
```

---

## GET `/api/auth/github`

Initiates the GitHub OAuth flow. Redirects to GitHub's authorization page requesting the `repo` scope.

**Required env vars:** `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`  
**Optional:** `GITHUB_REDIRECT_URL` (defaults to `http://127.0.0.1:3000/api/auth/callback`)

---

## GET `/api/auth/callback`

OAuth callback. Exchanges the authorization `code` for an access token, then returns a small HTML page that calls `window.opener.postMessage({ type: 'github-auth', token: '...' }, '*')` and closes the popup.

**Query parameters:** `code`, `state`  
**Success:** Returns HTML that passes the token back to the opener and closes the window.  
**Failure (bad code):** Posts `{ type: 'github-auth-error', error: '...' }`.  
**Failure (missing env var):** Returns a human-readable config error page — does not panic.

---

## GET `/api/auth/profile`

Fetches the authenticated user's profile statistics from the `atlas.db` SQLite database.

**Header Required:** `Authorization: Bearer <token>`
**Success `200`:**
```json
{
  "github_id": "1234567",
  "username": "Mithril64",
  "avatar_url": "https://avatars.githubusercontent.com/u/1234567?v=4",
  "commits": 5,
  "reviews": 2,
  "trust_rating": 3,
  "contribution_days": [
    { "date": "2026-03-11", "count": 2 },
    { "date": "2026-03-12", "count": 1 }
  ]
}
```
**Error `401`:**
```text
Unauthorized
```

---

## POST `/api/github/webhook`

Consumes GitHub webhook events and updates profile metrics.

**Headers:**
- `X-GitHub-Event` (required)
- `X-Hub-Signature-256` (required when `GITHUB_WEBHOOK_SECRET` is set)

**Supported events:**
- `pull_request` (`action=closed`, `merged=true`, `base.ref=main`) → increments `commits`
- `pull_request_review` (`action=submitted`) → increments `reviews`

Also writes daily contribution buckets in SQLite (`contributions` table), used by the profile heatmap.

**Success `200`:**
```json
{"status":"ok","counted":true}
```

---

## POST `/api/dev/replay-webhook`

Development-only helper to replay webhook payloads locally without GitHub delivery.

**Guard:** requires `ATLAS_ENABLE_DEV_WEBHOOK_REPLAY=true`.

**Headers:**
- `X-GitHub-Event` (required)

**Body:** raw JSON payload matching a GitHub webhook event shape.

**Success `200`:**
```json
{"status":"ok","counted":true,"dev":true}
```

---

## Notes

- CORS is `CorsLayer::permissive()` — fine for local use; restrict for production.
- Bind address is configurable: `SERVER_HOST` (default `127.0.0.1`), `SERVER_PORT` (default `3000`). `make server-public` sets `SERVER_HOST=0.0.0.0`.
- Submissions with `tags: [demo]` skip the Git/PR pipeline entirely.
- There is no rate limiting or file size limit currently implemented.
