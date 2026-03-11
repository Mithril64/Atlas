#theorem(
    id: "thm-pythagorean",
    deps: ("def-right-triangle",),
    tags: ("geometry", "euclidean",)
)[
#statement[
  In a right-angled triangle, the square of the length of the hypotenuse $c$ is equal to the sum of the squares of the lengths of the other two sides $a$ and $b$:
  
  $ a^2 + b^2 = c^2 $
]

#intuition[
  If you were to build a physical square extending outward from each side of a right triangle, the total area of the two smaller squares combined will perfectly equal the area of the largest square.
]

#proof[
  Let four identical right triangles with legs $a$, $b$, and hypotenuse $c$ be arranged to form a large square of side length $a+b$. 
  
  The area of this large square can be calculated in two ways:
  1. As the square of its side: $(a+b)^2 = a^2 + 2a b + b^2$
  2. As the sum of the four triangles and the inner square: $4(1/2 a b) + c^2 = 2a b + c^2$
  
  Equating the two expressions for the area:
  $ a^2 + 2a b + b^2 = 2a b + c^2 $
  
  Subtracting $2a b$ from both sides yields the desired result:
  $ a^2 + b^2 = c^2 $
]
]
