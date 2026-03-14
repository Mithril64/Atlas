// ==========================================
// ATLAS MATH GRAPH: CORE SCHEMA
// ==========================================

// 1. Top-Level Node Wrappers
// The Rust compiler automatically wraps the user's submission in one of these.
#let _atlas_node(type_name, badge_color, fill_color, stroke_color, id, deps, body) = block(
  width: 100%,
  inset: (top: 15pt, bottom: 15pt, left: 0pt, right: 0pt),
  [
    // Small floating badge for the node type
    #place(top + right, dy: -10pt)[
        #rect(
            fill: fill_color,
            stroke: 1pt + stroke_color,
            radius: 4pt,
            inset: (x: 6pt, y: 4pt)
        )[
            #text(fill: badge_color, size: 8pt, weight: "bold", tracking: 1pt)[#upper(type_name)]
        ]
    ]
    #body
  ]
)

// The exposed node types called by Rust
#let theorem(id: "", deps: (), tags: (), body) = _atlas_node("Theorem", rgb("#bd93f9"), rgb("#bd93f926"), rgb("#bd93f980"), id, deps, body)
#let lemma(id: "", deps: (), tags: (), body) = _atlas_node("Lemma", rgb("#8be9fd"), rgb("#8be9fd26"), rgb("#8be9fd80"), id, deps, body)
#let definition(id: "", deps: (), tags: (), body) = _atlas_node("Definition", rgb("#50fa7b"), rgb("#50fa7b26"), rgb("#50fa7b80"), id, deps, body)
#let axiom(id: "", deps: (), tags: (), body) = _atlas_node("Axiom", rgb("#ff5555"), rgb("#ff555526"), rgb("#ff555580"), id, deps, body)
#let intuition_node(id: "", deps: (), tags: (), body) = _atlas_node("Intuition", rgb("#f1fa8c"), rgb("#f1fa8c26"), rgb("#f1fa8c80"), id, deps, body)
#let proof_node(id: "", deps: (), tags: (), body) = _atlas_node("Proof", rgb("#ffb86c"), rgb("#ffb86c26"), rgb("#ffb86c80"), id, deps, body)

// ==========================================
// 2. Rigid Inner Blocks
// ==========================================
// Contributors must use these inside their submission body.

#let statement(body) = block(
  width: 100%,
  inset: (left: 14pt, top: 12pt, bottom: 12pt, right: 12pt),
  stroke: (left: 4pt + rgb("#bd93f9")), 
  fill: rgb("#bd93f91a"), 
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
  stroke: 1pt + rgb("#6272a480"),
  fill: rgb("#8be9fd0d"),
  radius: 4pt,
  below: 15pt,
  [
    #text(weight: "bold", fill: rgb("#f1fa8c"))[Intuition]
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

#let corollary(body) = block(
  width: 100%,
  inset: (left: 14pt, top: 12pt, bottom: 12pt, right: 12pt),
  stroke: (left: 4pt + rgb("#bd93f9")), 
  fill: rgb("#bd93f910"), 
  radius: (right: 4pt),
  below: 15pt,
  [
    #text(weight: "bold", size: 1.1em)[Corollary]
    #v(0.5em)
    #body
  ]
)

#let remark(body) = block(
  width: 100%,
  inset: (left: 14pt, top: 12pt, bottom: 12pt, right: 12pt),
  stroke: (left: 4pt + rgb("#6272a4")), 
  fill: rgb("#6272a41a"), 
  radius: (right: 4pt),
  below: 15pt,
  [
    #text(weight: "bold", style: "italic")[Remark.]
    #v(0.5em)
    #body
  ]
)

// ==========================================
// 3. Citation / Linking Macro
// ==========================================
// The Rust parser will cross-reference these against the `deps` array. [cite: 6, 7] (haha 67)
#let link_node(target_id) = {
  box(
    fill: rgb("#8be9fd26"),
    stroke: 1pt + rgb("#8be9fd80"),
    radius: 3pt,
    inset: (x: 4pt, y: 2pt)
  )[
    #text(fill: rgb("#8be9fd"), weight: "bold", size: 0.9em)[#target_id]
  ]
}
