# Atlas Contributor Quick Start Guide

Welcome to Atlas! This guide shows you how to submit your mathematical content to the graph.

## In 30 Seconds

1. Go to **https://atlas.timgerasimov.com/submit.html**
2. Create a `.typ` file with this format:
   ```
   // id: thm-my-theorem
   // type: theorem
   // deps: [def-group, ax-choice]
   ---
   
   Your Typst math here...
   ```
3. Drag & drop the file to upload
4. Done! Your contribution is live

---

## Submission Format

Every submission must start with three **metadata lines**:

```typst
// id: unique-identifier
// type: theorem
// deps: [optional-list-of-dependencies]
---

Your mathematical content in Typst...
```

### 1. ID (Required)

Unique identifier for your contribution. Use lowercase with hyphens:

**Good examples:**
- `thm-bolzano-weierstrass`
- `def-prime-number`
- `ax-axiom-of-choice`
- `lem-triangle-inequality`

**Avoid:**
- `Bolzano Weierstrass` (spaces)
- `theorem_1` (underscores)
- `THM-EXAMPLE` (uppercase)

**Format:** `{type-abbr}-{name}`

- `thm-` for theorems
- `def-` for definitions
- `ax-` for axioms
- `lem-` for lemmas
- `intuition-` for intuitions
- `proof-` for proofs

### 2. Type (Required)

One of these exactly:

| Type | Use For |
|------|---------|
| `theorem` | Main results and propositions |
| `lemma` | Helper lemmas (usually not famous) |
| `definition` | Formal definitions |
| `axiom` | Fundamental axioms |
| `intuition` | Informal explanations of ideas |
| `proof` | Detailed proofs |

### 3. Dependencies (Required)

List IDs of nodes this depends on. Can be empty:

```typst
// deps: []                           // No dependencies
// deps: [def-group]                  // Depends on one node
// deps: [def-group, ax-choice]       // Depends on two nodes
// deps: [thm-a, def-b, lemma-c]      // Mixed types
```

---

## Writing Math in Typst

Atlas uses **Typst** for mathematical content. You can find the reference [here](https://typst.app/docs/reference/syntax/)
### Functions

```typst
// Common functions
sin, cos, tan
log, ln, exp
lim, min, max
gcd, lcm

// Arrays/matrices
mat(
  1, 2;
  3, 4
)
```

### Full Example

```typst
*Theorem (Bolzano-Weierstrass):*

Every bounded sequence in $RR^n$ has a convergent subsequence.

_Proof sketch:_ Use the Heine-Borel compactness theorem...
```

See the [Typst Manual](https://typst.app/docs/) for complete reference.

---

## Validation Rules

Your submission will be **automatically validated** before being accepted. It must:

- [ ] Have exactly three metadata lines at the top (id, type, deps)
- [ ] Have valid Typst syntax (tested with `typst compile`)
- [ ] Have a unique ID (not already in the graph)
- [ ] Have a recognized type (theorem, lemma, definition, etc.)
- [ ] Have non-empty content after the `---` divider
- [ ] Use correct ID format (lowercase with hyphens)

If validation fails, you'll see a detailed error message.

---

## Troubleshooting

### "Missing 'id' metadata"

The first line doesn't contain `// id:`. Add it:

```typst
// id: thm-my-theorem
```

### "Invalid type 'Theorem'" 

Type must be lowercase. Change:

```typst
// type: theorem    Correct
// type: Theorem    Wrong
```

### "Typst compilation failed"

Your math syntax has an error. Common issues:

- Missing dollar signs: `x^2` should be `$x^2$`
- Wrong operator: `×` should be `times`
- Unmatched brackets: `$(x_n$ should be `$(x_n)$`

### "File size too large"

Keep submissions under 1 MB. Break very long proofs into multiple files.

### "Empty submission body"

Add content after the `---` divider:

```typst
// id: ...
// type: ...
// deps: ...
---

Your content goes here!
```

---

## Tips & Best Practices

1. **Reference related nodes**: Use meaningful dependencies so the graph is well-connected
2. **Modular structure**: Break long proofs into separate lemmas
3. **Use intuitions**: Add separate "intuition" nodes to explain the big ideas
4. **Consistent formatting**: Use Typst's built-in formatting for consistency
5. **Test locally**: Preview your submission before uploading
6. **Be rigorous**: Double-check math notation and logic

---

## Reference Material

For detailed information:

- **Typst Syntax**: https://typst.app/docs/
- **Atlas Architecture**: See `ARCHITECTURE.md` in the repo
- **Deployment**: See `DEPLOYMENT.md` for server configuration
- **System Checklist**: See `INTEGRATION_CHECKLIST.md`

---

## FAQ

**Q: Can I edit my submission after uploading?**
A: The file is saved to the repository. Contact an admin to edit or ask them to revert the commit and you can resubmit.

**Q: Can I depend on theorems I haven't proven yet?**
A: You can reference other nodes even if they're marked "TODO", but your math should be valid.

**Q: What if I make a mistake in the ID?**
A: It gets saved as a Git commit. An admin can revert it and you can resubmit with the correct ID.

**Q: Can I upload multiple files at once?**
A: Currently one at a time, but each submission is independent.

**Q: Is my submission peer-reviewed?**
A: Submissions are automatically validated for syntax. An admin may review for mathematical correctness.

---

## Next Steps

1. Write your first contribution
2. Visit `https://your-domain.com/submit.html`
3. Upload and see your node appear on the graph!
4. Share the link with others

Welcome to Atlas!
