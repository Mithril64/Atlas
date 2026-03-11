# Atlas API Reference

Base URL: `http://127.0.0.1:3000` (local development only — no hosted API)

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

**Success `200`** (non-demo submission):
```json
{"status": "success", "pr_url": "https://github.com/Mithril64/Atlas/pull/42"}
```

**Success `200`** (demo submission — `tags: [demo]`, skips Git):
```json
{"status": "success", "id": "thm-my-theorem"}
```

**Error `400`** — plain text from the Rust validation step:
```
No id
```
```
Validation failed: error: ...typst error...
```

**Error `500`:**
```
Git push failed
GITHUB_TOKEN not set
```

## GET `/api/auth/github`

Initiates the GitHub OAuth flow. Redirects the browser to GitHub's authorization page requesting the `public_repo` scope.

**Required environment variables:** `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`  
**Optional:** `GITHUB_REDIRECT_URL` (defaults to `http://127.0.0.1:3000/api/auth/callback`)

---

## GET `/api/auth/callback`

OAuth callback. Exchanges the GitHub authorization `code` for an access token, then returns a minimal HTML page that calls `window.opener.postMessage({ type: 'github-auth', token: '...' }, '*')` and closes the popup.

**Query parameters:** `code`, `state`  
**Success:** Returns HTML that passes the token back to the opener window.  
**Failure:** Returns HTML that posts `{ type: 'github-auth-error', error: '...' }`.

---

## Notes

- The server **does not** serve the static frontend — open `public/index.html` directly or use a separate static server.
- CORS is `CorsLayer::permissive()` — fine for local use, restrict for public deployment.
- **Authentication (OAuth):** If `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` are set, submissions will use the user's GitHub token if one is provided in the `Authorization: Bearer <token>` header. Falls back to `GITHUB_TOKEN` env var.
- There is no rate limiting or file size limit currently implemented.
