use std::fs;
use std::path::Path;
use std::process::Command;
use typst_syntax::{parse, SyntaxKind, SyntaxNode};
use serde::Serialize;
use regex::Regex;
use axum::{
    extract::Multipart,
    response::Json,
    routing::{post, get},
    Router,
};
use tower_http::cors::CorsLayer;
use std::env;

#[derive(Debug, Serialize)]
struct MathNode {
    id: String,
    node_type: String, 
    deps: Vec<String>,
    tags: Vec<String>, // Extracts tags for WebGL search filtering
    body: String, 
}

// Text extractor
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

// The AST Tree-Walker
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
            if child.kind() == SyntaxKind::Ident {
                let name = child.text().as_str();
                if name == "theorem" || name == "lemma" || name == "definition" || name == "axiom" || name == "intuition" {
                    is_math_node = true;
                    current_node.node_type = name.to_string();
                }
            }

            // We look inside the Args node...
            if is_math_node && child.kind() == SyntaxKind::Args {
                for arg in child.children() {
                    
                    if arg.kind() == SyntaxKind::Named {
                        let mut arg_name = String::new();
                        for part in arg.children() {
                            if part.kind() == SyntaxKind::Ident {
                                arg_name = part.text().to_string();
                            }
                            if arg_name == "id" && part.kind() == SyntaxKind::Str {
                                current_node.id = part.text().trim_matches('"').to_string();
                            }
                            if arg_name == "deps" && part.kind() == SyntaxKind::Array {
                                for array_item in part.children() {
                                    if array_item.kind() == SyntaxKind::Str {
                                        current_node.deps.push(array_item.text().trim_matches('"').to_string());
                                    }
                                }
                            }
                            if arg_name == "tags" && part.kind() == SyntaxKind::Array {
                                for array_item in part.children() {
                                    if array_item.kind() == SyntaxKind::Str {
                                        current_node.tags.push(array_item.text().trim_matches('"').to_string());
                                    }
                                }
                            }
                        }
                    }

                    if arg.kind() == SyntaxKind::ContentBlock {
                        let raw_text = extract_full_text(arg);
                        let trimmed = raw_text.strip_prefix('[').unwrap_or(&raw_text)
                                              .strip_suffix(']').unwrap_or(&raw_text);
                        current_node.body = trimmed.trim().to_string();
                    }
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
                    let root = parse(&source_code);
                    walk_tree(&root, all_nodes);
                }
            }
        }
    }
}

/// Parse loose submission format, extract metadata, and determine routing path
fn ingest_submission(raw_text: &str) -> Result<(String, String, String), String> {
    let re_id = Regex::new(r"//\s*id:\s*([^\n\r]+)").unwrap();
    let re_type = Regex::new(r"//\s*type:\s*([^\n\r]+)").unwrap();
    let re_deps = Regex::new(r"//\s*deps:\s*\[([^\]]*)\]").unwrap();
    let re_tags = Regex::new(r"//\s*tags:\s*\[([^\]]*)\]").unwrap();

    let id = re_id
        .captures(raw_text)
        .ok_or("Missing 'id' metadata (format: // id: name)")?[1]
        .trim()
        .to_string();

    let node_type = re_type
        .captures(raw_text)
        .ok_or("Missing 'type' metadata (format: // type: theorem|lemma|definition|axiom)")?[1]
        .trim()
        .to_string();

    let deps_raw = re_deps
        .captures(raw_text)
        .ok_or("Missing 'deps' metadata (format: // deps: [dep1, dep2])")?[1]
        .trim()
        .to_string();

    // Extract tags, defaulting to empty if missing
    let tags_raw = re_tags
        .captures(raw_text)
        .map_or("".to_string(), |cap| cap[1].trim().to_string());

    // Auto-Routing: Grab the first tag to use as the directory name
    let mut primary_tag = "uncategorized".to_string();
    if !tags_raw.is_empty() {
        if let Some(first_tag) = tags_raw.split(',').next() {
            let clean_tag = first_tag.trim().trim_matches(|c| c == '"' || c == '\'').to_lowercase().replace(" ", "-");
            if !clean_tag.is_empty() {
                primary_tag = clean_tag;
            }
        }
    }

    let valid_types = ["theorem", "lemma", "definition", "axiom", "intuition", "proof"];
    if !valid_types.contains(&node_type.as_str()) {
        return Err(format!("Invalid type '{}'. Must be one of: {}", node_type, valid_types.join(", ")));
    }

    let body = if let Some(pos) = raw_text.find("---") {
        raw_text[pos + 3..].trim().to_string()
    } else {
        raw_text.trim().to_string()
    };

    if body.is_empty() {
        return Err("Submission body is empty after frontmatter.".to_string());
    }

    let required_blocks = ["#statement", "#intuition", "#proof"];
    let mut missing_blocks = Vec::new();

    for block in required_blocks {
        if !body.contains(block) {
            missing_blocks.push(block);
        }
    }

    if !missing_blocks.is_empty() {
        return Err(format!(
            "Missing rigid blocks: {}. Every node must include a #statement[...], #intuition[...], and #proof[...] block.",
            missing_blocks.join(", ")
        ));
    }

    let formatted = format!(
        "#{}\n(\n    id: \"{}\",\n    deps: [{}],\n    tags: [{}]\n)[\n{}\n]\n",
        node_type, id, deps_raw, tags_raw, body
    );

    Ok((id, formatted, primary_tag))
}

/// Validate submission by attempting to compile with Typst
fn validate_submission(typst_code: &str, submission_id: &str) -> Result<(), String> {
    let temp_file = format!(".temp_validate_{}.typ", submission_id);
    let wrapper = format!(
        "#import \"../math/schema/math-graph.typ\": *\n\n{}",
        typst_code
    );

    fs::write(&temp_file, &wrapper)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let output = Command::new("typst")
        .args(["compile", "--root", "..", "--format", "pdf", &temp_file, "/dev/null"])
        .output()
        .map_err(|e| format!("Failed to execute typst: {}", e))?;

    let _ = fs::remove_file(&temp_file);

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Typst compilation failed: {}", stderr))
    }
}

fn main() {
    println!("Starting the atlas Compiler...");

    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "server" {
        println!("🚀 Starting atlas Submission Server...");
        start_server();
    } else {
        compile_all();
    }
}

#[tokio::main]
async fn start_server() {
    let app = Router::new()
        .route("/api/submit", post(upload_handler))
        .route("/api/graph", get(graph_handler))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("✓ Server running on http://127.0.0.1:3000");
    println!("  POST /api/submit  - Submit a new mathematical node");
    println!("  GET  /api/graph   - Fetch the current graph JSON");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn upload_handler(mut multipart: Multipart) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        if field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Read error: {}", e)))?;

            let content = String::from_utf8_lossy(&data).to_string();

            match ingest_submission(&content) {
                Ok((id, formatted_typst, primary_tag)) => {
                    // 1. Validate before saving
                    if let Err(validation_err) = validate_submission(&formatted_typst, &id) {
                        return Err((
                            axum::http::StatusCode::BAD_REQUEST,
                            format!("Validation failed: {}", validation_err),
                        ));
                    }

                    // 2. Create specific subdirectory based on the tag
                    let dir_path = format!("../math/{}", primary_tag);
                    fs::create_dir_all(&dir_path)
                        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create directory: {}", e)))?;

                    let path = format!("{}/{}.typ", dir_path, id);
                    fs::write(&path, &formatted_typst)
                        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write file: {}", e)))?;

                    // --- NEW: PREVENT DEMO SPAM ---
                    if primary_tag == "demo" {
                        return Ok(Json(serde_json::json!({
                            "status": "success",
                            "message": "Demo file processed and saved locally. Git/PR pipeline bypassed.",
                            "id": id
                        })));
                    }

                    // --- PR PIPELINE ---
                    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                    let branch_name = format!("submission-{}-{}", id, timestamp);

                    // Checkout new branch
                    Command::new("git").args(["checkout", "-b", &branch_name]).status().unwrap();

                    // Add and Commit
                    Command::new("git").args(["add", &path]).status().unwrap();
                    Command::new("git").args(["commit", "-m", &format!("atlas Submission: {}", id)]).status().unwrap();

                    // Push branch to remote
                    let push_status = Command::new("git").args(["push", "-u", "origin", &branch_name]).status();

                    // Checkout main branch again so the local server stays clean
                    Command::new("git").args(["checkout", "main"]).status().unwrap();

                    // Open the PR via GitHub API
                    if push_status.is_ok() && push_status.unwrap().success() {
                        match create_github_pr(&branch_name, &id).await {
                            Ok(pr_url) => {
                                return Ok(Json(serde_json::json!({
                                    "status": "success",
                                    "message": format!("Successfully pushed! PR opened at: {}", pr_url),
                                    "id": id,
                                    "pr_url": pr_url
                                })));
                            }
                            Err(e) => {
                                return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Pushed, but failed to open PR: {}", e)));
                            }
                        }
                    } else {
                        return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to push branch to GitHub. Check server permissions.".to_string()));
                    }
                }
                Err(parse_err) => {
                    return Err((
                        axum::http::StatusCode::BAD_REQUEST,
                        format!("Parsing error: {}", parse_err),
                    ));
                }
            }
        }
    }

    Err((
        axum::http::StatusCode::BAD_REQUEST,
        "No file field found in submission".to_string(),
    ))
}

async fn graph_handler() -> Json<serde_json::Value> {
    match fs::read_to_string("../public/json/graph.json") {
        Ok(content) => {
            if let Ok(json) = serde_json::from_str(&content) {
                return Json(json);
            }
        }
        Err(_) => {}
    }

    Json(serde_json::json!({
        "error": "Graph not yet compiled. Run: cargo run --release"
    }))
}

fn compile_all() {
    let math_dir = Path::new("../math");
    let mut all_nodes: Vec<MathNode> = Vec::new();
    
    process_directory(math_dir, &mut all_nodes);
    
    let json_output = serde_json::to_string_pretty(&all_nodes).unwrap();
    fs::write("../public/json/graph.json", &json_output).expect("Failed to write graph.json");
        
    println!("Compiling HTML fragments...");
    
    let nodes_dir = Path::new("../public/nodes");
    fs::create_dir_all(nodes_dir).expect("Failed to create public/nodes directory");

    for node in &all_nodes {
        if node.body.is_empty() { 
            continue; 
        } 

        // --- 1. Compile the Web SVG ---
        let svg_content = format!(
            "#import \"../math/schema/math-graph.typ\": *\n\
             #set page(width: 500pt, height: auto, margin: 10pt, fill: none)\n\
             #set text(fill: rgb(\"f8f8f2\"), size: 14pt)\n\n\
             {}", 
            node.body
        );
        let temp_svg = format!(".temp_svg_{}.typ", node.id);
        fs::write(&temp_svg, &svg_content).expect("Failed to write temp SVG file");

        let out_svg = format!("../public/nodes/{}.svg", node.id);
        Command::new("typst")
            .args(["compile", "--root", "..", &temp_svg, &out_svg])
            .status()
            .expect("Failed to execute Typst CLI for SVG.");
            
        let _ = fs::remove_file(&temp_svg);

        // --- 2. Compile the Downloadable PDF ---
        let pdf_content = format!(
            "#import \"../math/schema/math-graph.typ\": *\n\
             #set page(width: auto, height: auto, margin: 20pt, fill: rgb(\"ffffff\"))\n\
             #set text(fill: rgb(\"000000\"), size: 12pt)\n\n\
             {}", 
            node.body
        );
        let temp_pdf = format!(".temp_pdf_{}.typ", node.id);
        fs::write(&temp_pdf, &pdf_content).expect("Failed to write temp PDF file");

        let out_pdf = format!("../public/nodes/{}.pdf", node.id);
        let pdf_status = Command::new("typst")
            .args(["compile", "--root", "..", &temp_pdf, &out_pdf])
            .status()
            .expect("Failed to execute Typst CLI for PDF.");

        if pdf_status.success() {
            println!("Compiled SVG & PDF: {}", node.id);
        } else {
            eprintln!("Failed to compile PDF for: {}", node.id);
        }
        
        let _ = fs::remove_file(&temp_pdf);
    }

    println!("atlas Compilation Successful!");
}

async fn create_github_pr(branch_name: &str, node_id: &str) -> Result<String, String> {
    let token = env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN not set in environment".to_string())?;
    
    let repo_owner = "Mithril64";
    let repo_name = "Atlas"; // Left capitalized since it represents the target URL
    
    let url = format!("https://api.github.com/repos/{}/{}/pulls", repo_owner, repo_name);

    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "title": format!("atlas Submission: {}", node_id),
        "body": format!("Automated submission for node `{}` from the atlas web portal.", node_id),
        "head": branch_name,
        "base": "main"
    });

    let response = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "atlas-Compiler-Bot")
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let res_json: serde_json::Value = response.json().await.map_err(|_| "Failed to parse JSON".to_string())?;
        let pr_url = res_json["html_url"].as_str().unwrap_or("Unknown URL").to_string();
        Ok(pr_url)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("GitHub API Error: {}", error_text))
    }
}
