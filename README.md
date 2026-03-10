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
typst compile math/algebra/group.typ
```

### For Developers (Building the Engine)
You will need Rust and Typst CLI installed.

**Step 1: Compile the graph**
```bash
cd Atlas/compiler
cargo run --release
```
This generates `../public/json/graph.json` and SVG/PDF fragments.

**Step 2: Start the web server**
```bash
cd Atlas/compiler
cargo run --release -- server
```
This starts the submission API on `http://127.0.0.1:3000`.

**Step 3: Open the web interface**
```bash
# Main graph viewer
open http://127.0.0.1:3000/../public/index.html

# Contributor portal (submit new nodes)
open http://127.0.0.1:3000/../public/submit.html
```

---

## Contributing

### For Mathematicians & Researchers

Submit new definitions, theorems, and proofs via the web portal:

1. Go to **`/submit.html`** on the Atlas server
2. Write your content in Typst with metadata:
   ```typst
   // id: thm-my-theorem
   // type: theorem
   // deps: [def-group, ax-choice]
   ---
   
   Your mathematical content here...
   ```
3. Upload and your contribution is automatically validated and added to the graph!

See [**CONTRIBUTOR_GUIDE.md**](CONTRIBUTOR_GUIDE.md) for detailed instructions.

### For Software Developers

* **Backend (Rust):** Improve the parser, add validation, optimize compilation
* **Frontend (HTML/JS):** Enhance the graph viewer, submission UI, search functionality
* **DevOps:** Help with deployment, CI/CD, monitoring

See [**DEPLOYMENT.md**](DEPLOYMENT.md) and [**ARCHITECTURE.md**](ARCHITECTURE.md) for technical details.

---

## Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and data flow
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production deployment guide
- **[API_REFERENCE.md](API_REFERENCE.md)** - REST API endpoints
- **[CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md)** - How to submit content
- **[INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md)** - Implementation checklist

---

## License

This project is open-source and licensed under the MIT License.
