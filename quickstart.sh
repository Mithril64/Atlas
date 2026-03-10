#!/bin/bash

# Atlas Quick Start Script
# This script automates the setup and running of Atlas

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║          Atlas Quick Start Script          ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
    echo
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

check_requirements() {
    print_step "Checking requirements..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Install from https://rustup.rs/"
        exit 1
    fi
    print_success "Rust is installed ($(cargo --version))"
    
    if ! command -v typst &> /dev/null; then
        print_error "Typst CLI not found. Install with: cargo install typst-cli"
        exit 1
    fi
    print_success "Typst is installed ($(typst --version))"
    
    if ! command -v git &> /dev/null; then
        print_error "Git not found"
        exit 1
    fi
    print_success "Git is installed"
}

build_compiler() {
    print_step "Building the compiler..."
    
    cd compiler
    cargo build --release 2>&1 | tail -5
    
    if [ -f target/release/math-graph-compiler ]; then
        print_success "Compiler built successfully"
    else
        print_error "Build failed"
        exit 1
    fi
    
    cd ..
}

compile_graph() {
    print_step "Compiling the math graph..."
    
    cd compiler
    cargo run --release > /dev/null 2>&1
    
    if [ -f ../public/json/graph.json ]; then
        local node_count=$(grep -o '"id"' ../public/json/graph.json | wc -l)
        print_success "Graph compiled with $node_count nodes"
    else
        print_error "Graph compilation failed"
        exit 1
    fi
    
    cd ..
}

start_server() {
    print_step "Starting Atlas server..."
    echo
    echo -e "${YELLOW}╔════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║           Server is running!               ║${NC}"
    echo -e "${YELLOW}╠════════════════════════════════════════════╣${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  📊 Graph Viewer:                          ║${NC}"
    echo -e "${YELLOW}║     http://127.0.0.1:3000/../public/       ║${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  📝 Submit Portal:                         ║${NC}"
    echo -e "${YELLOW}║     http://127.0.0.1:3000/../public/submit.html ║${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  🔌 API Endpoints:                         ║${NC}"
    echo -e "${YELLOW}║     POST /api/submit - Upload submissions  ║${NC}"
    echo -e "${YELLOW}║     GET  /api/graph  - Fetch graph data    ║${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  Press Ctrl+C to stop the server           ║${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}╚════════════════════════════════════════════╝${NC}"
    echo
    
    cd compiler
    cargo run --release -- server
}

run_demo() {
    print_step "Running submission demo..."
    
    # Create a demo submission
    local demo_file="demo-submission.typ"
    
    cat > "$demo_file" << 'EOF'
// id: thm-demo-example
// type: theorem
// deps: []
---

This is a demonstration submission created by the quick start script.

*Theorem:* For any two real numbers $a$ and $b$:

$ (a + b)^2 = a^2 + 2ab + b^2 $

*Proof:* Expand the left side directly by multiplication.
EOF
    
    print_info "Created demo file: $demo_file"
    
    # Try to submit it (server should be running)
    print_step "Attempting to submit demo file..."
    
    if command -v curl &> /dev/null; then
        local response=$(curl -s -X POST \
            -F "file=@$demo_file" \
            http://127.0.0.1:3000/api/submit)
        
        if echo "$response" | grep -q "success"; then
            print_success "Demo submission successful!"
            print_info "Check math/submissions/thm-demo-example.typ"
        else
            print_info "Could not submit (is server running?)"
            print_info "Try manually at: http://127.0.0.1:3000/../public/submit.html"
        fi
    fi
    
    rm "$demo_file"
}

show_help() {
    print_header
    echo "Usage: $0 [COMMAND]"
    echo
    echo "Commands:"
    echo "  build      Build the compiler (required first time)"
    echo "  compile    Compile the math graph"
    echo "  server     Start the web server"
    echo "  demo       Run a demo submission"
    echo "  full       Do everything: build, compile, and start server"
    echo "  help       Show this help message"
    echo
    echo "Examples:"
    echo "  $0 full       # First-time setup and start server"
    echo "  $0 server     # Start server (after initial setup)"
    echo "  $0 build      # Just build the compiler"
    echo
}

main() {
    print_header
    
    local cmd="${1:-help}"
    
    case "$cmd" in
        build)
            check_requirements
            build_compiler
            print_success "Build complete!"
            ;;
        compile)
            compile_graph
            print_success "Compilation complete!"
            ;;
        server)
            start_server
            ;;
        demo)
            print_info "Starting demo requires a running server"
            run_demo
            ;;
        full)
            check_requirements
            build_compiler
            compile_graph
            start_server
            ;;
        help)
            show_help
            ;;
        *)
            print_error "Unknown command: $cmd"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
