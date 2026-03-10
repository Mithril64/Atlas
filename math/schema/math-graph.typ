// ==========================================
// ATLAS MATH GRAPH: CORE SCHEMA
// ==========================================

// 1. Top-Level Node Wrappers
// The Rust compiler automatically wraps the user's submission in one of these.
#let _atlas_node(type_name, border_color, id, deps, body) = block(
  width: 100%,
  inset: (top: 15pt, bottom: 15pt, left: 0pt, right: 0pt),
  [
    // Small floating badge for the node type
    #place(top + right, dy: -10pt)[
        #rect(
            fill: border_color.transparentize(85%),
            stroke: 1pt + border_color.transparentize(50%),
            radius: 4pt,
            inset: (x: 6pt, y: 4pt)
        )[
            #text(fill: border_color, size: 8pt, weight: "bold", tracking: 1pt)[#type_name.upper()]
        ]
    ]
    #body
  ]
)

// The exposed node types called by Rust
#let theorem(id: "", deps: (), tags: (), body) = _atlas_node("Theorem", rgb("#bd93f9"), id, deps, body)
#let lemma(id: "", deps: (), tags: (), body) = _atlas_node("Lemma", rgb("#8be9fd"), id, deps, body)
#let definition(id: "", deps: (), tags: (), body) = _atlas_node("Definition", rgb("#50fa7b"), id, deps, body)
#let axiom(id: "", deps: (), tags: (), body) = _atlas_node("Axiom", rgb("#ff5555"), id, deps, body)
#let intuition_node(id: "", deps: (), tags: (), body) = _atlas_node("Intuition", rgb("#f1fa8c"), id, deps, body)
#let proof_node(id: "", deps: (), tags: (), body) = _atlas_node("Proof", rgb("#ffb86c"), id, deps, body)

// ==========================================
// 2. Rigid Inner Blocks
// ==========================================
// Contributors must use these inside their submission body.

#let statement(body) = block(
  width: 100%,
  inset: (left: 14pt, top: 12pt, bottom: 12pt, right: 12pt),
  stroke: (left: 4pt + rgb("#bd93f9")), 
  fill: rgb("#bd93f9").transparentize(90%), 
  radius: (right: 4pt),
  below: 15pt,
  [
    #text(weight: "bold", size: 1.1em)[Statement]
    #v(0.5em)
    #body
  ]
)

#let intuition(body) = block(
  width: 100%,
  inset: (left: 14pt, top: 12pt, bottom: 12pt, right: 12pt),
  stroke: 1pt + rgb("#6272a4").transparentize(50%),
  fill: rgb("#8be9fd").transparentize(95%),
  radius: 4pt,
  below: 15pt,
  [
    #text(weight: "bold", fill: rgb("#f1fa8c"))[💡 Intuition]
    #v(0.5em)
    #body
  ]
)

#let proof(body) = block(
  width: 100%,
  inset: (left: 14pt, top: 12pt, bottom: 12pt, right: 12pt),
  below: 10pt,
  [
    #text(style: "italic", weight: "bold")[Proof.]
    #v(0.5em)
    #body
    #h(1fr) $square$
  ]
)

// ==========================================
// 3. Citation / Linking Macro
// ==========================================
// The Rust parser will cross-reference these against the `deps` array. [cite: 6, 7] (haha 67)
#let link_node(target_id) = {
  box(
    fill: rgb("#8be9fd").transparentize(85%),
    stroke: 1pt + rgb("#8be9fd").transparentize(50%),
    radius: 3pt,
    inset: (x: 4pt, y: 2pt)
  )[
    #text(fill: rgb("#8be9fd"), weight: "bold", size: 0.9em)[#target_id]
  ]
}
