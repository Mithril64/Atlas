#!/bin/bash

# atlas Quick Start Script
# This script automates the setup and running of atlas

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║          atlas Quick Start Script          ║${NC}"
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

check_github_token() {
    print_step "Checking for GitHub API token..."
    if [ -z "$GITHUB_TOKEN" ]; then
        print_error "GITHUB_TOKEN environment variable is not set!"
        echo
        echo -e "${YELLOW}To use the automated PR pipeline, you need a GitHub Personal Access Token.${NC}"
        echo "1. Go to https://github.com/settings/tokens"
        echo "2. Click 'Generate new token (classic)'"
        echo "3. Give it a note (e.g., 'atlas Bot') and check the 'repo' scope."
        echo "4. Copy the token and export it in your terminal:"
        echo
        echo -e "   ${BLUE}export GITHUB_TOKEN=\"ghp_your_secret_token_here\"${NC}"
        echo
        echo "After exporting the token, run this script again."
        exit 1
    fi
    print_success "GitHub token is configured"
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
    print_step "Starting atlas server..."
    echo
    echo -e "${YELLOW}╔════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║           Server is running!               ║${NC}"
    echo -e "${YELLOW}╠════════════════════════════════════════════╣${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  🖥️  Frontend (Run in a separate terminal): ║${NC}"
    echo -e "${YELLOW}║     cd public && python3 -m http.server    ║${NC}"
    echo -e "${YELLOW}║     Graph: http://localhost:8000           ║${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  🔌 API Endpoints (Listening on 3000):     ║${NC}"
    echo -e "${YELLOW}║     POST /api/submit - Upload submissions  ║${NC}"
    echo -e "${YELLOW}║     GET  /api/graph  - Fetch graph data    ║${NC}"
    echo -e "${YELLOW}║                                            ║${NC}"
    echo -e "${YELLOW}║  Press Ctrl+C to stop the Rust backend     ║${NC}"
    echo -e "${YELLOW}╚════════════════════════════════════════════╝${NC}"
    echo
    
    cd compiler
    cargo run --release -- server
}

run_demo() {
    print_step "Running submission demo..."
    
    local demo_file="demo-submission.typ"
    
    # Updated to match our rigid format constraints and routing tags!
    cat > "$demo_file" << 'EOF'
// id: thm-demo-example
// type: theorem
// deps: []
// tags: ["demo"]
---

#statement[
    For any two real numbers $a$ and $b$, $(a + b)^2 = a^2 + 2a b + b^2$.
]

#intuition[
    This is the basic geometric expansion of a square with sides $a+b$.
]

#proof[
    Expand the left side directly by multiplication.
]
EOF
    
    print_info "Created demo file: $demo_file"
    print_step "Attempting to submit demo file..."
    
    if command -v curl &> /dev/null; then
        local response=$(curl -s -X POST \
            -F "file=@$demo_file" \
            http://127.0.0.1:3000/api/submit)
        
        if echo "$response" | grep -q "success"; then
            print_success "Demo submission successful!"
            print_info "Check math/demo/thm-demo-example.typ"
        else
            print_error "Server rejected it. Did you start the server first?"
            print_info "Response: $response"
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
	    check_github_token
            start_server
            ;;
        demo)
            print_info "Starting demo requires a running server"
            run_demo
            ;;
        full)
            check_requirements
	    check_github_token
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
