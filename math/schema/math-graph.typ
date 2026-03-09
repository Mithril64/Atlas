// ==========================================
// THE MATH GRAPH: CORE SCHEMA
// ==========================================
// This file defines the strict functions required for the 
// Rust compiler to extract the mathematical dependency graph.
// ==========================================

// 1. Base Block Constructor (Internal Use Only)
// This creates the visual boxes for the mathematicians viewing the PDFs.
#let _math_block(type_name, color, title, body) = block(
  fill: color.lighten(90%),
  stroke: (left: 4pt + color),
  inset: 12pt,
  width: 100%,
  radius: (right: 4pt),
  [
    #text(weight: "bold", fill: color.darken(20%))[#type_name]
    #if title != none [
      #text(weight: "bold")[ (#title)]
    ]
    #v(0.5em)
    #body
  ]
)

// ==========================================
// 2. Core Graph Nodes (Axioms, Defs, Lemmas, Theorems)
// ==========================================

#let axiom(id: "", name: none, tags: (), body) = {
  // The Rust parser will look for the `axiom` function and extract `id`
  _math_block("Axiom", rgb("#e74c3c"), name, body)
}

#let definition(id: "", name: none, tags: (), deps: (), body) = {
  _math_block("Definition", rgb("#f39c12"), name, body)
}

#let lemma(id: "", name: none, tags: (), deps: (), status: "draft", body) = {
  _math_block("Lemma", rgb("#3498db"), name, body)
}

#let theorem(id: "", name: none, tags: (), deps: (), status: "draft", body) = {
  _math_block("Theorem", rgb("#9b59b6"), name, body)
}

// ==========================================
// 3. Attached Content (Proofs & Intuition)
// ==========================================
// These don't create new nodes in the DAG; they attach to existing nodes.

#let proof(for_id: "", body) = block(
  inset: (left: 12pt, top: 8pt, bottom: 8pt),
  [
    #text(style: "italic", weight: "bold")[Proof.]
    #body
    #h(1fr) $square$
  ]
)

#let intuition(for_id: "", body) = block(
  fill: rgb("#ecf0f1"),
  inset: 12pt,
  radius: 4pt,
  width: 100%,
  [
    #text(weight: "bold", fill: rgb("#7f8c8d"))[💡 Intuition]
    #v(0.5em)
    #body
  ]
)

// ==========================================
// 4. Citation / Linking Macro
// ==========================================
// Use this to reference other nodes in the text. The Rust parser will 
// also cross-reference these to ensure they exist in the `deps` array.

#let link_node(target_id) = {
  text(fill: rgb("#2980b9"), weight: "bold")[#target_id]
}
