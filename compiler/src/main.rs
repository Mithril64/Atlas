use std::fs;
use std::path::Path;
use typst_syntax::{parse, SyntaxKind, SyntaxNode};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct MathNode {
    id: String,
    node_type: String, 
    deps: Vec<String>,
}

// AST tree walker
fn walk_tree(node: &SyntaxNode, extracted_nodes: &mut Vec<MathNode>) {
    if node.kind() == SyntaxKind::FuncCall {
        let mut is_math_node = false;
        let mut current_node = MathNode {
            id: String::new(),
            node_type: String::new(),
            deps: Vec::new(),
        };
        
        for child in node.children() {
            if child.kind() == SyntaxKind::Ident {
                let name = child.text().as_str();
                if name == "theorem" || name == "lemma" || name == "definition" || name == "axiom" {
                    is_math_node = true;
                    current_node.node_type = name.to_string();
                }
            }

            if is_math_node && child.kind() == SyntaxKind::Args {
                for arg in child.children() {
                    if arg.kind() == SyntaxKind::Named {
                        let mut arg_name = String::new();
                        for part in arg.children() {
                            if part.kind() == SyntaxKind::Ident {
                                arg_name = part.text().to_string();
                            }
                            if arg_name == "id" && part.kind() == SyntaxKind::Str {
                                let raw_str = part.text().trim_matches('"');
                                current_node.id = raw_str.to_string();
                            }
                            if arg_name == "deps" && part.kind() == SyntaxKind::Array {
                                for array_item in part.children() {
                                    if array_item.kind() == SyntaxKind::Str {
                                        let raw_str = array_item.text().trim_matches('"');
                                        current_node.deps.push(raw_str.to_string());
                                    }
                                }
                            }
                        }
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

// 2. The Directory Crawler
fn process_directory(dir: &Path, all_nodes: &mut Vec<MathNode>) {
    if dir.is_dir() {
        // Read the contents of the directory
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // If it's the schema folder, ignore it
                    if path.ends_with("schema") {
                        continue;
                    }
                    process_directory(&path, all_nodes);
                } else {
                    if path.extension().and_then(|s| s.to_str()) == Some("typ") {
                        println!("Parsing file: {:?}", path);
                        
                        let source_code = fs::read_to_string(&path)
                            .expect("Failed to read the Typst file.");
                            
                        let root = parse(&source_code);
                        walk_tree(&root, all_nodes);
                    }
                }
            }
        }
    }
}

// 3. The Main Execution
fn main() {
    println!("Starting the Math Graph Compiler...");

    // Point the crawler at the root math directory
    let math_dir = Path::new("../math");
    let mut all_nodes: Vec<MathNode> = Vec::new();
    
    // Start the recursive crawl
    process_directory(math_dir, &mut all_nodes);
    
    // Convert everything into JSON
    let json_output = serde_json::to_string_pretty(&all_nodes).unwrap();
    
    // Save it to the public directory
    let output_path = "../public/graph.json";
    fs::write(output_path, &json_output)
        .expect("Failed to write graph.json. Does the public/ directory exist?");
        
    println!("Compilation Successful!");
    println!("Extracted {} math nodes from the directory and saved them to '{}'", all_nodes.len(), output_path);
}
