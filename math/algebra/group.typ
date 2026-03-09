#include "../schema/math-graph.typ": *

#definition(
  id: "def-group",
  name: "Group",
  tags: ("algebra", "group-theory"),
  deps: ("def-set", "def-binary-operation")
)[
  A group is a set $G$ equipped with a binary operation $*$ that satisfies closure, associativity, identity, and invertibility.
]


#intuition(for_id: "def-group")[
  Think of a group as a collection of symmetries. For example, all the ways you can rotate or flip a square without changing its overall footprint form a group!
]


#axiom(

  id: "ax-group-associativity",
  name: "Group Associativity",
  tags: ("algebra", "group-theory"),
  deps: ("def-group")

)[
  For all $a, b, c in G$, the equation $(a * b) * c = a * (b * c)$ holds valid.
]
