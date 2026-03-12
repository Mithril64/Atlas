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

Initiates the GitHub OAuth flow. Redirects to GitHub's authorization page requesting the `public_repo` scope.

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

## Notes

- CORS is `CorsLayer::permissive()` — fine for local use; restrict for production.
- Bind address is configurable: `SERVER_HOST` (default `127.0.0.1`), `SERVER_PORT` (default `3000`). `make server-public` sets `SERVER_HOST=0.0.0.0`.
- Submissions with `tags: [demo]` skip the Git/PR pipeline entirely.
- There is no rate limiting or file size limit currently implemented.
