use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use typst_syntax::{parse, SyntaxKind, SyntaxNode};
use serde::Serialize;
use axum::{
    extract::Multipart,
    extract::State,
    response::Json,
    routing::{get, post},
    http::{HeaderMap, StatusCode},
    Router,
};
use axum::http::HeaderName;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use std::env;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use regex::Regex;
use math_graph_compiler::ingest_submission;

mod auth;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppState {
    webhook_secret: Option<String>,
}

#[derive(Debug, Serialize)]
struct MathNode {
    id: String,
    node_type: String, 
    deps: Vec<String>,
    tags: Vec<String>, 
    body: String, 
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
}

fn render_wikilinks(body: &str, base: &str) -> String {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    re.replace_all(body, |caps: &regex::Captures| {
        let id = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let text = caps.get(2).map(|m| m.as_str().trim()).filter(|s| !s.is_empty()).unwrap_or(id);
        format!("#link(\"{}/#{}\")[{}]", base.trim_end_matches('/'), id, text)
    }).into_owned()
}

fn extract_proof_blocks(body: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut i = 0usize;
    while let Some(rel) = body[i..].find("#proof[") {
        let open = i + rel;
        let mut depth = 0usize;
        let mut start_idx = None;
        let mut end_idx = None;

        for (offset, ch) in body[open..].char_indices() {
            let idx = open + offset;
            match ch {
                '[' => {
                    depth += 1;
                    if depth == 1 {
                        start_idx = Some(idx + 1);
                    }
                }
                ']' => {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = Some(idx);
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        if let (Some(start), Some(end)) = (start_idx, end_idx) {
            blocks.push(body[start..end].to_string());
            i = end + 1;
        } else {
            break;
        }
    }
    blocks
}

fn extract_wikilink_ids(text: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    let mut set = HashSet::new();
    for cap in re.captures_iter(text) {
        if let Some(id) = cap.get(1).map(|m| m.as_str().trim()) {
            if !id.is_empty() {
                set.insert(id.to_string());
            }
        }
    }
    set.into_iter().collect()
}

fn extract_full_text(node: &SyntaxNode) -> String {
    let mut result = String::new();
    if !node.text().is_empty() {
        result.push_str(node.text());
    }
    for child in node.children() {
        result.push_str(&extract_full_text(child));
    }
    result
}

// Robust AST Walker
fn walk_tree(node: &SyntaxNode, extracted_nodes: &mut Vec<MathNode>) {
    if node.kind() == SyntaxKind::FuncCall {
        let mut is_math_node = false;
        let mut current_node = MathNode {
            id: String::new(),
            node_type: String::new(),
            deps: Vec::new(),
            tags: Vec::new(),
            body: String::new(),
        };
        
        for child in node.children() {
            // 1. Identify the node type (e.g., #theorem)
            if child.kind() == SyntaxKind::Ident {
                let name = child.text().as_str();
                if ["theorem", "lemma", "definition", "axiom", "intuition", "proof"].contains(&name) {
                    is_math_node = true;
                    current_node.node_type = name.to_string();
                }
            }

            // 2. Extract Metadata and Body
            if is_math_node {
                // Check inside Args (...)
                if child.kind() == SyntaxKind::Args {
                    for arg in child.children() {
                        if arg.kind() == SyntaxKind::Named {
                            let mut arg_name = String::new();
                            for part in arg.children() {
                                if part.kind() == SyntaxKind::Ident { arg_name = part.text().to_string(); }
                                if arg_name == "id" && part.kind() == SyntaxKind::Str {
                                    current_node.id = part.text().trim_matches('"').to_string();
                                }
                                if arg_name == "deps" && part.kind() == SyntaxKind::Array {
                                    for item in part.children() {
                                        if item.kind() == SyntaxKind::Str { current_node.deps.push(item.text().trim_matches('"').to_string()); }
                                    }
                                }
                                if arg_name == "tags" && part.kind() == SyntaxKind::Array {
                                    for item in part.children() {
                                        if item.kind() == SyntaxKind::Str { current_node.tags.push(item.text().trim_matches('"').to_string()); }
                                    }
                                }
                            }
                        }
                        // Handle body if it's inside the args: #func([body])
                        if arg.kind() == SyntaxKind::ContentBlock {
                            let raw = extract_full_text(arg);
                            current_node.body = raw.strip_prefix('[').unwrap_or(&raw).strip_suffix(']').unwrap_or(&raw).trim().to_string();
                        }
                    }
                }
                
                // Handle body if it's a trailing block: #func()[body]
                if child.kind() == SyntaxKind::ContentBlock {
                    let raw = extract_full_text(child);
                    current_node.body = raw.strip_prefix('[').unwrap_or(&raw).strip_suffix(']').unwrap_or(&raw).trim().to_string();
                }
            }
        }
        
        if is_math_node && !current_node.id.is_empty() {
            extracted_nodes.push(current_node);
        }
    }

    for child in node.children() {
        walk_tree(child, extracted_nodes);
    }
}

fn process_directory(dir: &Path, all_nodes: &mut Vec<MathNode>) {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.ends_with("schema") || path.ends_with("submissions") { continue; }
                    process_directory(&path, all_nodes);
                } else if path.extension().and_then(|s| s.to_str()) == Some("typ") {
                    let source_code = fs::read_to_string(&path).expect("Failed to read file.");
                    println!("  🔍 Scanning: {:?}", path.file_name().unwrap());

                    // If ingestion fails (missing tags/deps), fall back to raw parsing
                    if let Ok((_, formatted, _)) = ingest_submission(&source_code) {
                        walk_tree(&parse(&formatted), all_nodes);
                    } else {
                        walk_tree(&parse(&source_code), all_nodes);
                    }
                }
            }
        }
    }
}

fn init_db() {
    let conn = rusqlite::Connection::open("atlas.db").expect("Failed to open DB");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            github_id TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            avatar_url TEXT,
            commits INTEGER DEFAULT 0,
            reviews INTEGER DEFAULT 0,
            trust_rating INTEGER DEFAULT 1
        )",
        [],
    ).expect("Failed to create users table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contributions (
            github_id TEXT NOT NULL,
            day TEXT NOT NULL,
            kind TEXT NOT NULL,
            count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (github_id, day, kind)
        )",
        [],
    ).expect("Failed to create contributions table");
}

fn main() {
    let mut public_loaded = false;
    if std::env::var("SERVER_HOST").as_deref() == Ok("0.0.0.0") || std::env::var("DOTENV_FILE").as_deref() == Ok(".env.public") {
        if let Ok(content) = std::fs::read_to_string(".env.public") {
            for line in content.lines() {
                let trimmed = line.trim();
                // ignore comments and empty lines
                if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
                if let Some((k, v)) = trimmed.split_once('=') {
                    std::env::set_var(k.trim(), v.trim());
                }
            }
            public_loaded = true;
            println!("Loaded .env.public config (overriding cargo defaults)");
        }
    }
    
    // If public config wasn't requested or failed to load, load local .env.
    // We don't load both because dotenv() does not overwrite existing vars,
    // so loading .env after .env.public would be safe but confusing if they mix.
    if !public_loaded {
        let dotenv_file = std::env::var("DOTENV_FILE").unwrap_or_else(|_| ".env".to_string());
        if dotenv::from_filename(&dotenv_file).is_ok() {
            println!("Loaded {} config", dotenv_file);
        }
    }
    println!("Starting the atlas Compiler...");
    init_db();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "server" {
        start_server();
    } else {
        compile_all();
    }
}

#[tokio::main]
async fn start_server() {
    ensure_compiled_assets();

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            HeaderName::from_static("ngrok-skip-browser-warning"),
        ]);

    let app_state = AppState {
        webhook_secret: env::var("GITHUB_WEBHOOK_SECRET").ok(),
    };

    let app = Router::new()
        .nest("/api/auth", auth::auth_router())
        .route("/api/submit", post(upload_handler))
        .route("/api/graph", get(graph_handler))
        .merge(
            Router::new()
                .route("/api/github/webhook", post(github_webhook_handler))
                .route("/api/dev/replay-webhook", post(dev_replay_webhook_handler))
                .with_state(app_state)
        )
        .nest_service("/nodes", ServeDir::new(repo_root().join("public/nodes")))
        .nest_service("/json", ServeDir::new(repo_root().join("public/json")))
        .nest_service("/api/nodes", ServeDir::new(repo_root().join("public/nodes")))
        .nest_service("/api/json", ServeDir::new(repo_root().join("public/json")))
        .layer(cors);
    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));
    println!("✓ Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn upload_handler(headers: axum::http::HeaderMap, mut multipart: Multipart) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let auth_token = headers.get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());
    while let Some(field) = multipart.next_field().await.map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;
            let content = String::from_utf8_lossy(&data).to_string();
            match ingest_submission(&content) {
                Ok((id, formatted, tag)) => {
                    let dir = repo_root().join("math").join(&tag);
                    fs::create_dir_all(&dir).unwrap();
                    let path = dir.join(format!("{}.typ", id));
                    fs::write(&path, &formatted).unwrap();
                    
                    if tag == "demo" { return Ok(Json(serde_json::json!({"status": "success", "id": id}))); }
                    
                    let branch = format!("submission-{}-{}", id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                    let checkout = Command::new("git").current_dir(repo_root()).args(["checkout", "-b", &branch]).output();
                    let checkout = match checkout {
                        Ok(o) if o.status.success() => o,
                        Ok(o) => {
                            return Err((
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!(
                                    "Git checkout failed: {}{}",
                                    String::from_utf8_lossy(&o.stderr),
                                    String::from_utf8_lossy(&o.stdout)
                                )
                            ));
                        }
                        Err(e) => return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Git checkout execution failed: {}", e))),
                    };
                    let _ = checkout;

                    let add = Command::new("git").current_dir(repo_root()).args(["add", path.to_str().unwrap_or_default()]).output();
                    match add {
                        Ok(o) if o.status.success() => {}
                        Ok(o) => {
                            let _ = Command::new("git").current_dir(repo_root()).args(["checkout", "main"]).output();
                            return Err((
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!(
                                    "Git add failed: {}{}",
                                    String::from_utf8_lossy(&o.stderr),
                                    String::from_utf8_lossy(&o.stdout)
                                )
                            ));
                        }
                        Err(e) => {
                            let _ = Command::new("git").current_dir(repo_root()).args(["checkout", "main"]).output();
                            return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Git add execution failed: {}", e)));
                        }
                    }

                    let commit = Command::new("git").current_dir(repo_root()).args(["commit", "-m", &format!("atlas Submission: {}", id)]).output();
                    match commit {
                        Ok(o) if o.status.success() => {}
                        Ok(o) => {
                            let _ = Command::new("git").args(["checkout", "main"]).output();
                            return Err((
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!(
                                    "Git commit failed: {}{}",
                                    String::from_utf8_lossy(&o.stderr),
                                    String::from_utf8_lossy(&o.stdout)
                                )
                            ));
                        }
                        Err(e) => {
                            let _ = Command::new("git").args(["checkout", "main"]).output();
                            return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Git commit execution failed: {}", e)));
                        }
                    }

                    let push = Command::new("git").current_dir(repo_root()).args(["push", "-u", "origin", &branch]).output();
                    let _ = Command::new("git").current_dir(repo_root()).args(["checkout", "main"]).output();

                    if let Ok(push_out) = push {
                        if push_out.status.success() {
                        let pr_url = create_github_pr(&branch, &id, auth_token.as_deref()).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

                        return Ok(Json(serde_json::json!({"status": "success", "pr_url": pr_url})));
                        }

                        return Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!(
                                "Git push failed: {}{}",
                                String::from_utf8_lossy(&push_out.stderr),
                                String::from_utf8_lossy(&push_out.stdout)
                            )
                        ));
                    }
                    return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Git push execution failed".to_string()));
                }
                Err(e) => return Err((axum::http::StatusCode::BAD_REQUEST, e)),
            }
        }
    }
    Err((axum::http::StatusCode::BAD_REQUEST, "No file".to_string()))
}

fn record_contribution(github_id: &str, username: &str, avatar_url: &str, kind: &str) -> Result<(), String> {
    let conn = rusqlite::Connection::open("atlas.db").map_err(|e| e.to_string())?;
    let day = chrono::Utc::now().date_naive().to_string();

    conn.execute(
        "INSERT INTO users (github_id, username, avatar_url, commits, reviews)
         VALUES (?1, ?2, ?3, 0, 0)
         ON CONFLICT(github_id) DO UPDATE SET
            username = excluded.username,
            avatar_url = excluded.avatar_url",
        [github_id, username, avatar_url],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO contributions (github_id, day, kind, count)
         VALUES (?1, ?2, ?3, 1)
         ON CONFLICT(github_id, day, kind) DO UPDATE SET count = count + 1",
        [github_id, day.as_str(), kind],
    ).map_err(|e| e.to_string())?;

    match kind {
        "merge" => {
            conn.execute("UPDATE users SET commits = commits + 1 WHERE github_id = ?1", [github_id])
                .map_err(|e| e.to_string())?;
        }
        "review" => {
            conn.execute("UPDATE users SET reviews = reviews + 1 WHERE github_id = ?1", [github_id])
                .map_err(|e| e.to_string())?;
        }
        _ => {}
    }

    Ok(())
}

fn process_webhook_event(event: &str, payload: &serde_json::Value) -> Result<bool, String> {
    if event == "pull_request" {
        let action = payload["action"].as_str().unwrap_or_default();
        let merged = payload["pull_request"]["merged"].as_bool().unwrap_or(false);
        let base_ref = payload["pull_request"]["base"]["ref"].as_str().unwrap_or_default();
        if action == "closed" && merged && base_ref == "main" {
            if let Some(merger) = payload["pull_request"]["merged_by"].as_object() {
                let github_id = merger.get("id").and_then(|v| v.as_i64()).map(|v| v.to_string()).unwrap_or_default();
                let username = merger.get("login").and_then(|v| v.as_str()).unwrap_or("unknown");
                let avatar = merger.get("avatar_url").and_then(|v| v.as_str()).unwrap_or("");
                if !github_id.is_empty() {
                    record_contribution(&github_id, username, avatar, "merge")?;
                    return Ok(true);
                }
            }
        }
    }

    if event == "pull_request_review" {
        let action = payload["action"].as_str().unwrap_or_default();
        if action == "submitted" {
            if let Some(reviewer) = payload["review"]["user"].as_object() {
                let github_id = reviewer.get("id").and_then(|v| v.as_i64()).map(|v| v.to_string()).unwrap_or_default();
                let username = reviewer.get("login").and_then(|v| v.as_str()).unwrap_or("unknown");
                let avatar = reviewer.get("avatar_url").and_then(|v| v.as_str()).unwrap_or("");
                if !github_id.is_empty() {
                    record_contribution(&github_id, username, avatar, "review")?;
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

fn verify_signature(secret: &str, body: &[u8], signature_header: Option<&str>) -> bool {
    let Some(signature_header) = signature_header else {
        return false;
    };
    let Some(sig_hex) = signature_header.strip_prefix("sha256=") else {
        return false;
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let computed = hex::encode(mac.finalize().into_bytes());
    computed.eq_ignore_ascii_case(sig_hex)
}

async fn github_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let event = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    if event.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing X-GitHub-Event header".to_string()));
    }

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok());

    if let Some(secret) = &state.webhook_secret {
        if !verify_signature(secret, &body, signature) {
            return Err((StatusCode::UNAUTHORIZED, "Invalid webhook signature".to_string()));
        }
    }

    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON payload: {}", e)))?;
    let counted = process_webhook_event(&event, &payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({"status": "ok", "counted": counted})))
}

async fn dev_replay_webhook_handler(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let enabled = env::var("ATLAS_ENABLE_DEV_WEBHOOK_REPLAY").unwrap_or_else(|_| "false".to_string());
    if enabled != "true" {
        return Err((StatusCode::FORBIDDEN, "Dev webhook replay is disabled".to_string()));
    }

    let event = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    if event.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing X-GitHub-Event header".to_string()));
    }

    let counted = process_webhook_event(&event, &payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"status": "ok", "counted": counted, "dev": true})))
}

async fn graph_handler() -> Json<serde_json::Value> {
    let path = repo_root().join("public/json/graph.json");
    let content = fs::read_to_string(&path).unwrap_or_default();
    Json(serde_json::from_str(&content).unwrap_or(serde_json::json!({"error": "No graph"})))
}

fn ensure_compiled_assets() {
    let graph_path = repo_root().join("public/json/graph.json");
    let nodes_dir = repo_root().join("public/nodes");
    let nodes_present = nodes_dir.read_dir().map(|mut r| r.next().is_some()).unwrap_or(false);

    if graph_path.exists() && nodes_present {
        return;
    }

    println!("Graph assets missing or empty; running compile_all() before starting server...");
    compile_all();
}

fn compile_all() {
    let root = repo_root();
    let tmp_dir = root.clone();
    let link_base = env::var("ATLAS_LINK_BASE").unwrap_or_else(|_| "https://atlasmath.org".to_string());

    let mut all_nodes = Vec::new();
    process_directory(&root.join("math"), &mut all_nodes);

    // Auto-add wikilink targets inside proof blocks to dependency lists
    for node in all_nodes.iter_mut() {
        let mut deps_set: HashSet<String> = node.deps.iter().cloned().collect();
        for block in extract_proof_blocks(&node.body) {
            for id in extract_wikilink_ids(&block) {
                if deps_set.insert(id.clone()) {
                    node.deps.push(id);
                }
            }
        }
    }

    let graph_path = root.join("public/json/graph.json");
    fs::create_dir_all(graph_path.parent().unwrap()).unwrap();
    fs::write(&graph_path, serde_json::to_string_pretty(&all_nodes).unwrap()).expect("Write failed");
    
    println!("Compiling HTML fragments...");
    let nodes_dir = root.join("public/nodes");
    fs::create_dir_all(nodes_dir).unwrap();

    if all_nodes.is_empty() {
        println!("  ⚠️ No nodes found! Check your .typ files for proper #theorem(...) formatting.");
    }

    for node in all_nodes {
        if node.body.is_empty() { continue; }
        
        let rendered_body = render_wikilinks(&node.body, &link_base);

        let svg_content = format!(
            "#import \"public/math-graph.typ\": *\n#set page(width: 500pt, height: auto, margin: 10pt, fill: none)\n#set text(fill: rgb(\"f8f8f2\"), size: 14pt)\n\n{}",
            rendered_body
        );
        let temp_svg = tmp_dir.join(format!(".temp_{}.typ", node.id));
        fs::write(&temp_svg, &svg_content).unwrap();
        let svg_out = Command::new("typst")
            .current_dir(&root)
            .args([
                "compile",
                "--root",
                root.to_str().unwrap_or(".."),
                temp_svg.to_str().unwrap_or_default(),
                root.join("public/nodes").join(format!("{}.svg", node.id)).to_str().unwrap_or_default()
            ])
            .output()
            .unwrap();
        if !svg_out.status.success() {
            eprintln!(
                "SVG compile failed for {}: {}{}",
                node.id,
                String::from_utf8_lossy(&svg_out.stderr),
                String::from_utf8_lossy(&svg_out.stdout)
            );
            let _ = fs::remove_file(&temp_svg);
            panic!("SVG compile failed for {}", node.id);
        }
        let _ = fs::remove_file(&temp_svg);

        let pdf_content = format!(
            "#import \"public/math-graph.typ\": *\n#set page(width: 595pt, height: auto, margin: (x: 56pt, y: 48pt), fill: rgb(\"#282a36\"))\n#set text(fill: rgb(\"#f8f8f2\"), size: 12pt)\n\n{}",
            rendered_body
        );
        let temp_pdf = tmp_dir.join(format!(".temp_pdf_{}.typ", node.id));
        fs::write(&temp_pdf, &pdf_content).unwrap();
        let pdf_out = Command::new("typst")
            .current_dir(&root)
            .args([
                "compile",
                "--root",
                root.to_str().unwrap_or(".."),
                temp_pdf.to_str().unwrap_or_default(),
                root.join("public/nodes").join(format!("{}.pdf", node.id)).to_str().unwrap_or_default()
            ])
            .output()
            .unwrap();
        if !pdf_out.status.success() {
            eprintln!(
                "PDF compile failed for {}: {}{}",
                node.id,
                String::from_utf8_lossy(&pdf_out.stderr),
                String::from_utf8_lossy(&pdf_out.stdout)
            );
            let _ = fs::remove_file(&temp_pdf);
            panic!("PDF compile failed for {}", node.id);
        }
        println!("  ✓ Compiled: {}", node.id);
        let _ = fs::remove_file(&temp_pdf);
    }
    println!("atlas Compilation Successful!");
}

async fn create_github_pr(branch: &str, id: &str, user_token: Option<&str>) -> Result<String, String> {
    let token = user_token
        .map(|s| s.to_string())
        .or_else(|| env::var("GITHUB_TOKEN").ok())
        .ok_or_else(|| "No GitHub token — log in with GitHub to submit".to_string())?;
    let url = "https://api.github.com/repos/Mithril64/Atlas/pulls";
    let client = reqwest::Client::new();
    let payload = serde_json::json!({"title": format!("atlas Submission: {}", id), "body": "Automated submission.", "head": branch, "base": "main"});
    let res = client.post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "atlas-bot")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let body = res.text().await.unwrap_or_default();

    if status.is_success() {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        Ok(json["html_url"].as_str().unwrap_or_default().to_string())
    } else {
        // Try to extract a human-readable message from GitHub's JSON error body
        let msg = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|j| j["message"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| body.clone());
        eprintln!("GitHub PR creation failed: HTTP {} — {}", status, body);
        Err(format!("GitHub API error {}: {}", status.as_u16(), msg))
    }
}
