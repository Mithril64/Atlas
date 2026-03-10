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

#[derive(Debug, Serialize)]
struct MathNode {
    id: String,
    node_type: String, 
    deps: Vec<String>,
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

/// Parse loose submission format and extract metadata
fn ingest_submission(raw_text: &str) -> Result<(String, String), String> {
    // Regex patterns to find frontmatter comments
    let re_id = Regex::new(r"//\s*id:\s*([^\n\r]+)").unwrap();
    let re_type = Regex::new(r"//\s*type:\s*([^\n\r]+)").unwrap();
    let re_deps = Regex::new(r"//\s*deps:\s*\[([^\]]*)\]").unwrap();

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

    let deps_raw_capture = re_deps
        .captures(raw_text)
        .ok_or("Missing 'deps' metadata (format: // deps: [dep1, dep2])")?;
    let deps_raw = deps_raw_capture[1].trim().to_string();

    // Validate node type
    let valid_types = ["theorem", "lemma", "definition", "axiom", "intuition", "proof"];
    if !valid_types.contains(&node_type.as_str()) {
        return Err(format!(
            "Invalid type '{}'. Must be one of: {}",
            node_type,
            valid_types.join(", ")
        ));
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
        "#{}\n(\n    id: \"{}\",\n    deps: [{}]\n)[\n{}\n]\n",
        node_type, id, deps_raw, body
    );

    Ok((id, formatted))
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
        .args(["compile", "--root", "..", &temp_file, "/dev/null"])
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
    println!("Starting the Atlas Compiler...");

    let args: Vec<String> = std::env::args().collect();
    
    // Check if running in server mode
    if args.len() > 1 && args[1] == "server" {
        println!("🚀 Starting Atlas Submission Server...");
        start_server();
    } else {
        // Default: Compile mode
        compile_all();
    }
}

#[tokio::main]
async fn start_server() {
    // Build our application with routes
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
                Ok((id, formatted_typst)) => {
                    // Validate before saving
                    if let Err(validation_err) = validate_submission(&formatted_typst, &id) {
                        return Err((
                            axum::http::StatusCode::BAD_REQUEST,
                            format!("Validation failed: {}", validation_err),
                        ));
                    }

                    // Create submissions directory if it doesn't exist
                    let submissions_dir = Path::new("../math/submissions");
                    fs::create_dir_all(submissions_dir)
                        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create directory: {}", e)))?;

                    // Save file to /math/submissions/
                    let path = format!("../math/submissions/{}.typ", id);
                    fs::write(&path, &formatted_typst)
                        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write file: {}", e)))?;

                    // Automated Git Commit
                    let git_add = Command::new("git")
                        .args(["add", &path])
                        .status();

                    if let Err(e) = git_add {
                        eprintln!("Warning: git add failed: {}", e);
                    }

                    let git_commit = Command::new("git")
                        .args(["commit", "-m", &format!("Atlas Submission: {}", id)])
                        .status();

                    match git_commit {
                        Ok(status) if status.success() => {
                            return Ok(Json(serde_json::json!({
                                "status": "success",
                                "message": format!("Successfully submitted and committed: {}", id),
                                "id": id
                            })));
                        }
                        Ok(_) => {
                            // Commit might fail if nothing changed, but file was written
                            return Ok(Json(serde_json::json!({
                                "status": "success",
                                "message": format!("File saved: {} (git commit skipped)", id),
                                "id": id
                            })));
                        }
                        Err(e) => {
                            return Err((
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Git error: {}", e),
                            ));
                        }
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

        // --- 1. Compile the Web SVG (Dark Mode, Transparent Background) ---
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

        // --- 2. Compile the Downloadable PDF (Light Mode, Clean Document) ---
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

    println!("Atlas Compilation Successful!");
}
