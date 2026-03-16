#theorem(
    id: "thm-wikilink-test",
    deps: ["def-group"],
    tags: ["algebra", "wikilinks"]
)[
#statement[
  A small regression-test node to exercise wikilink rendering and auto-dependency capture from proofs.
]

#intuition[
  This theorem is intentionally trivial; its value is in linking to other nodes via Obsidian-style syntax.
]

#proof[
  We reference a base definition [[def-group|group axioms]] that is already in deps and a linked theorem [[thm-pythagorean]] that is not. The compiler should add the latter automatically when scanning proof blocks.

  Therefore, after compilation, this node should depend on both `def-group` and `thm-pythagorean` and render the links as clickable.
]
]
