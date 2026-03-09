# Atlas

An open-source, community-driven visual map of mathematics. 

Atlas represents modern theorems, lemmas, and definitions as a massive, interactive Directed Acyclic Graph (DAG). Powered by a purely static architecture, it acts as both a rigorous mathematical encyclopedia and a local, deterministic proof-writing environment.

[Image of an interactive WebGL network graph representing mathematical theorems and their dependencies (WIP)]

---

## The Vision

Mathematical knowledge is currently locked inside dense, linear PDFs. This makes it difficult for students and researchers to see the "big picture" or trace the fundamental dependencies of a complex theorem. 

Atlas solves this by parsing human-readable Typst files into a structured dependency tree. It provides: 
1. **Rigorous Proofs:** Step-by-step formal verifications.
2. **Pedagogical Intuition:** Plain-English, geometric, or conceptual explanations for every node.
3. **Explorable Connections:** A clean visual interface to traverse the tree from modern research all the way down to foundational axioms.

## Core Features

* **Interactive WebGL DAG:** A lightning-fast, explorable map of all mathematical dependencies.
* **Deterministic "Proof Copilot":** A local, browser-based Typst editor that uses classic link-prediction algorithms (e.g., Adamic-Adar) to suggest logical next steps and relevant lemmas based on your current citations. **(Strictly zero LLMs or generative AI).**
* **Zero-Cost Infrastructure:** The entire platform compiles down to static HTML and a single `graph.json` database, allowing it to be hosted globally via CDNs for free.

---

## Tech Stack

This project is built on a "hypermedia-driven" philosophy to ensure extreme scalability and simplicity:

* **Data Layer:** [Typst](https://typst.app/) (A modern, lightning-fast alternative to LaTeX).
* **Compiler:** Rust (Parses the Typst AST and generates the static site and JSON graph).
* **Frontend:** HTMX (for instantaneous content loading) and WebGL (for graph rendering).

For a deep dive into how the system works, please read our [Architecture Specification](ARCHITECTURE.md).

---

## Repository Structure

Our monorepo is divided strictly by discipline to ensure a frictionless experience for both mathematicians and developers:

* `/math` - The source of truth. Pure `.typ` files containing theorems, proofs, and intuition. (Mathematicians live here).
* `/compiler` - The Rust CLI tool that parses the Typst AST and builds the database. (Backend developers live here).
* `/frontend` - The static HTML, HTMX, and WebGL rendering logic. (Web developers live here).

---

## Getting Started (Local Development)

### For Mathematicians (Writing Content)
You only need the Typst CLI installed to start writing and previewing math.
```bash
# Clone the repository
git clone https://github.com/Mithril64/Atlas.git
cd Atlas

# Compile a specific file to PDF for local viewing
typst compile math/analysis/real/bolzano.typ
```

### For Developers (Building the Engine)
You will need `cargo` (Rust) and `npm`/`yarn` installed.
```bash
# Clone the repository
git clone https://github.com/Mithril64/Atlas.git
cd Atlas

# Run the Rust parser to generate the public/ HTML and JSON files
cd compiler
cargo run -- --source ../math --output ../public

# Serve the static frontend locally
cd ../frontend
npx serve
```

---

## Contributing


We are actively looking for contributors across all disciplines! 
* **Mathematicians:** Help us map out new subfields, write intuition blocks, and verify proofs.
* **Rust Developers:** Help us optimize the AST traversal and metadata extraction.
* **Frontend Developers:** Help us build a beautiful, performant WebGL canvas and WASM editor.


Please read our [Contributing Guidelines](CONTRIBUTING.md) to get started.

---

## License

This project is open-source and licensed under the MIT License.
