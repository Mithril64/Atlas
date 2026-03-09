use std::fs;
use std::path::Path;
use std::process::Command;
use typst_syntax::{parse, SyntaxKind, SyntaxNode};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct MathNode {
    id: String,
    node_type: String, 
    deps: Vec<String>,
    #[serde(skip)] 
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
                    if path.ends_with("schema") { continue; }
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

fn main() {
    println!("Starting the Atlas Compiler...");

    let math_dir = Path::new("../math");
    let mut all_nodes: Vec<MathNode> = Vec::new();
    
    process_directory(math_dir, &mut all_nodes);
    
    let json_output = serde_json::to_string_pretty(&all_nodes).unwrap();
    fs::write("../public/json/graph.json", &json_output).expect("Failed to write graph.json");
        
    println!("Compiling HTML fragments...");
    
    let nodes_dir = Path::new("../public/nodes");
    fs::create_dir_all(nodes_dir).expect("Failed to create public/nodes directory");

    for node in &all_nodes {
        // If the file is just an empty block like #theorem(id: "test")[], skip it
        if node.body.is_empty() { 
            continue; 
        } 

        // 1. Create a temporary Typst file formatted specifically for the Web UI
        let temp_content = format!(
            "#import \"../math/schema/math-graph.typ\": *\n\
             #set page(width: 400pt, height: auto, margin: 10pt, fill: none)\n\
             #set text(fill: rgb(\"f8f8f2\"), size: 14pt)\n\n\
             {}", 
            node.body
        );
        let temp_file = format!(".temp_{}.typ", node.id);
        fs::write(&temp_file, &temp_content).expect("Failed to write temp file");

        // 2. Command the Typst CLI to compile it to SVG instead of HTML
        let out_svg = format!("../public/nodes/{}.svg", node.id); // <--- Changed to .svg
        let status = Command::new("typst")
            .args([
                "compile", 
                "--root", "..", // Keep the root permission!
                &temp_file, 
                &out_svg
            ]) // <--- Removed the experimental --features html flag
            .status()
            .expect("Failed to execute Typst CLI. Is it installed and in your PATH?");

        if status.success() {
            println!("  ↳ Compiled: {}.svg", node.id);
        } else {
            eprintln!("  ❌ Failed to compile: {}", node.id);
        }

        // 3. Clean up the temporary file
        let _ = fs::remove_file(&temp_file);
    }

    println!("Atlas Compilation Successful!");
}
