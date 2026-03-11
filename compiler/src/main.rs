use std::fs;
use std::path::Path;
use std::process::Command;
use typst_syntax::{parse, SyntaxKind, SyntaxNode};
use serde::Serialize;
use axum::{
    extract::Multipart,
    response::Json,
    routing::{post, get},
    Router,
};
use tower_http::cors::CorsLayer;
use std::env;
use math_graph_compiler::ingest_submission;

mod auth;

#[derive(Debug, Serialize)]
struct MathNode {
    id: String,
    node_type: String, 
    deps: Vec<String>,
    tags: Vec<String>, 
    body: String, 
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


fn main() {
    // Load .env.public when running in public mode, .env otherwise.
    // Override by setting DOTENV_FILE=path before launching.
    let dotenv_file = std::env::var("DOTENV_FILE").unwrap_or_else(|_| {
        if std::env::var("SERVER_HOST").as_deref() == Ok("0.0.0.0") {
            ".env.public".to_string()
        } else {
            ".env".to_string()
        }
    });
    dotenv::from_filename(&dotenv_file).ok();
    dotenv::dotenv().ok(); // fallback: also load .env so local vars still work
    println!("Starting the atlas Compiler...");
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "server" {
        start_server();
    } else {
        compile_all();
    }
}

#[tokio::main]
async fn start_server() {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    let app = Router::new()
        .nest("/api/auth", auth::auth_router())
        .route("/api/submit", post(upload_handler))
        .route("/api/graph", get(graph_handler))
        .layer(CorsLayer::permissive());
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
                    let dir = format!("../math/{}", tag);
                    fs::create_dir_all(&dir).unwrap();
                    let path = format!("{}/{}.typ", dir, id);
                    fs::write(&path, &formatted).unwrap();
                    
                    if tag == "demo" { return Ok(Json(serde_json::json!({"status": "success", "id": id}))); }
                    
                    let branch = format!("submission-{}-{}", id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                    Command::new("git").args(["checkout", "-b", &branch]).status().unwrap();
                    Command::new("git").args(["add", &path]).status().unwrap();
                    Command::new("git").args(["commit", "-m", &format!("atlas Submission: {}", id)]).status().unwrap();
                    let push = Command::new("git").args(["push", "-u", "origin", &branch]).status();
                    Command::new("git").args(["checkout", "main"]).status().unwrap();

                    if push.is_ok() && push.unwrap().success() {
                        let pr_url = create_github_pr(&branch, &id, auth_token.as_deref()).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
                        return Ok(Json(serde_json::json!({"status": "success", "pr_url": pr_url})));
                    }
                    return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Git push failed".to_string()));
                }
                Err(e) => return Err((axum::http::StatusCode::BAD_REQUEST, e)),
            }
        }
    }
    Err((axum::http::StatusCode::BAD_REQUEST, "No file".to_string()))
}

async fn graph_handler() -> Json<serde_json::Value> {
    let content = fs::read_to_string("../public/json/graph.json").unwrap_or_default();
    Json(serde_json::from_str(&content).unwrap_or(serde_json::json!({"error": "No graph"})))
}

fn compile_all() {
    let mut all_nodes = Vec::new();
    process_directory(Path::new("../math"), &mut all_nodes);
    fs::write("../public/json/graph.json", serde_json::to_string_pretty(&all_nodes).unwrap()).expect("Write failed");
    
    println!("Compiling HTML fragments...");
    let nodes_dir = Path::new("../public/nodes");
    fs::create_dir_all(nodes_dir).unwrap();

    if all_nodes.is_empty() {
        println!("  ⚠️ No nodes found! Check your .typ files for proper #theorem(...) formatting.");
    }

    for node in all_nodes {
        if node.body.is_empty() { continue; }
        
        let svg_content = format!("#import \"../math/schema/math-graph.typ\": *\n#set page(width: 500pt, height: auto, margin: 10pt, fill: none)\n#set text(fill: rgb(\"f8f8f2\"), size: 14pt)\n\n{}", node.body);
        let temp_svg = format!(".temp_{}.typ", node.id);
        fs::write(&temp_svg, &svg_content).unwrap();
        Command::new("typst").args(["compile", "--root", "..", &temp_svg, &format!("../public/nodes/{}.svg", node.id)]).status().unwrap();
        let _ = fs::remove_file(&temp_svg);

        let pdf_content = format!("#import \"../math/schema/math-graph.typ\": *\n#set page(width: auto, height: auto, margin: 20pt, fill: white)\n#set text(fill: black, size: 12pt)\n\n{}", node.body);
        let temp_pdf = format!(".temp_pdf_{}.typ", node.id);
        fs::write(&temp_pdf, &pdf_content).unwrap();
        let status = Command::new("typst").args(["compile", "--root", "..", &temp_pdf, &format!("../public/nodes/{}.pdf", node.id)]).status().unwrap();
        if status.success() { println!("  ✓ Compiled: {}", node.id); }
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
