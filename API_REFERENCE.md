# Atlas API Reference

Complete API documentation for the Atlas math graph system.

---

## Base URL

```
http://127.0.0.1:3000    (local development)
https://atlas.timgerasimov.com  (production)
```

---

## Endpoints

### 1. GET `/api/graph`

**Description**: Fetch the current mathematical graph

**Method**: `GET`

**Query Parameters**: None

**Response**: JSON object containing all nodes and edges

**Example**:

```bash
curl http://127.0.0.1:3000/api/graph
```

**Response (200 OK)**:

```json
[
  {
    "id": "thm-bolzano-weierstrass",
    "node_type": "theorem",
    "deps": ["def-bounded-sequence", "thm-monotone-subsequence"],
    "body": "Every bounded sequence in ℝⁿ..."
  },
  {
    "id": "def-bounded-sequence",
    "node_type": "definition",
    "deps": [],
    "body": "A sequence (xₙ) is bounded if..."
  }
]
```

**Error Response (500 Internal Server Error)**:

```json
{
  "error": "Graph not yet compiled. Run: cargo run --release"
}
```

---

### 2. POST `/api/submit`

**Description**: Submit a new mathematical node

**Method**: `POST`

**Content-Type**: `multipart/form-data`

**Request Body**:

- `file` (form field): A `.typ` file with Typst content

**Example**:

```bash
curl -X POST \
  -F "file=@submission.typ" \
  http://127.0.0.1:3000/api/submit
```

**Success Response (200 OK)**:

```json
{
  "status": "success",
  "message": "Successfully submitted and committed: thm-my-theorem",
  "id": "thm-my-theorem"
}
```

**Validation Error (400 Bad Request)**:

```json
{
  "status": "error",
  "message": "Parsing error: Missing 'id' metadata (format: // id: name)"
}
```

```json
{
  "status": "error",
  "message": "Validation failed: Typst compilation failed: undefined variable 'x'"
}
```

**Server Error (500 Internal Server Error)**:

```json
{
  "status": "error",
  "message": "Failed to write file: Permission denied"
}
```

---

## Error Codes

| Code | Meaning | Resolution |
|------|---------|-----------|
| 200 | Success | Submission accepted |
| 400 | Bad Request | Fix metadata or Typst syntax |
| 500 | Server Error | Check server logs |

---

## Submission Format

The file sent to `/api/submit` must contain:

```typst
// id: unique-id
// type: theorem|lemma|definition|axiom|intuition|proof
// deps: [dep1, dep2, ...]
---

Your Typst mathematical content here...
```

### Metadata Requirements

| Field | Required | Format | Example |
|-------|----------|--------|---------|
| `id` | Yes | `// id: {value}` | `// id: thm-bolzano` |
| `type` | Yes | `// type: {theorem\|lemma\|definition\|axiom\|intuition\|proof}` | `// type: theorem` |
| `deps` | Yes | `// deps: [id1, id2, ...]` | `// deps: [def-limit, ax-reals]` |

---

## Client Integration

### JavaScript (Fetch API)

```javascript
// Upload a submission
async function submitNode(file) {
  const formData = new FormData();
  formData.append('file', file);

  const response = await fetch('/api/submit', {
    method: 'POST',
    body: formData
  });

  const data = await response.json();
  
  if (response.ok) {
    console.log('Success:', data.message);
  } else {
    console.error('Error:', data.message);
  }
}

// Fetch the graph
async function loadGraph() {
  const response = await fetch('/api/graph');
  const nodes = await response.json();
  
  return nodes;
}
```

### Python (Requests)

```python
import requests

# Upload a submission
def submit_node(file_path):
    with open(file_path, 'rb') as f:
        files = {'file': f}
        response = requests.post('http://127.0.0.1:3000/api/submit', files=files)
    
    return response.json()

# Fetch the graph
def load_graph():
    response = requests.get('http://127.0.0.1:3000/api/graph')
    return response.json()
```

### cURL (Command Line)

```bash
# Upload a file
curl -X POST \
  -F "file=@theorem.typ" \
  http://127.0.0.1:3000/api/submit | jq '.'

# Fetch the graph
curl http://127.0.0.1:3000/api/graph | jq '.'
```

---

## CORS Policy

The server currently uses **permissive CORS** for development:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
```

**For production**, restrict CORS to your domain:

```rust
let cors = CorsLayer::very_permissive()
    .allow_origin("https://atlas.yourdomain.com".parse().unwrap());
```

---

## Rate Limiting

Currently **not implemented**. For production, add rate limiting:

```bash
# Limit to 1 request per minute per IP
curl -X POST \
  -H "X-Rate-Limit-Key: your-api-key" \
  -F "file=@submission.typ" \
  https://atlas.yourdomain.com/api/submit
```

---

## WebSocket API (Future)

Planned for real-time graph updates:

```javascript
const ws = new WebSocket('ws://127.0.0.1:3000/api/graph/stream');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Graph updated:', update);
};
```

---

## File Upload Limits

- **Max file size**: Currently unlimited (should add 10 MB limit in production)
- **Allowed extensions**: `.typ` only
- **Content encoding**: UTF-8 required

---

## Authentication (Future)

Planned authentication schemes:

```bash
# Bearer token
curl -H "Authorization: Bearer token123" \
  http://127.0.0.1:3000/api/submit

# API key
curl -H "X-API-Key: key123" \
  http://127.0.0.1:3000/api/submit
```

---

## Response Timing

Typical response times:

| Operation | Time |
|-----------|------|
| GET `/api/graph` | < 100 ms |
| POST `/api/submit` (valid) | 200-500 ms |
| POST `/api/submit` (Typst validation) | 1-3 sec |
| POST `/api/submit` (Git commit) | 500-1000 ms |

---

## Webhooks (Future)

Planned webhook events:

```
POST https://your-webhook-url.com/hooks/atlas

Events:
- submission.received
- submission.validated
- submission.saved
- submission.committed
- graph.updated
```

Payload:

```json
{
  "event": "submission.saved",
  "timestamp": "2026-03-10T15:30:45Z",
  "data": {
    "id": "thm-new-theorem",
    "type": "theorem",
    "dependencies": ["def-x", "def-y"]
  }
}
```

---

## Monitoring & Health Checks

```bash
# Health check endpoint (planned)
curl http://127.0.0.1:3000/health

# Response:
# {"status": "ok", "uptime": 3600, "graph_nodes": 42}
```

---

## Debugging

### Enable Request Logging

Set environment variable before starting:

```bash
RUST_LOG=debug cargo run --release -- server
```

### Common Issues

**CORS error in browser**:
```
Access to XMLHttpRequest blocked by CORS policy
```
→ Server's CORS settings need to be updated

**Connection refused**:
```
curl: (7) Failed to connect
```
→ Make sure server is running: `cargo run --release -- server`

**Invalid submission rejected**:
```
Parsing error: Missing 'type' metadata
```
→ Check metadata format matches documentation exactly

---

## Backwards Compatibility

The API version is **v0.1** (unreleased). Breaking changes may occur.

To future-proof your client:

```javascript
const API_VERSION = 'v1';
const baseUrl = `/api/${API_VERSION}`;

// When v2 is released, update to:
// const API_VERSION = 'v2';
```

---

## Support

For issues, questions, or feature requests:

1. Check `CONTRIBUTOR_GUIDE.md` for submission help
2. See `DEPLOYMENT.md` for configuration
3. Review `ARCHITECTURE.md` for system design
4. File an issue on GitHub (when ready)

---

**Last Updated**: March 10, 2026
**API Version**: v0.1 (Development)
**Status**: Fully Functional
