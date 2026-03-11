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

---

## Notes

- The server **does not** serve the static frontend files — open `public/index.html` directly or use a separate static server.
- CORS is set to `CorsLayer::permissive()` — fine for local use, should be restricted for any public deployment.
- There is no authentication, rate limiting, or file size limit currently implemented.
