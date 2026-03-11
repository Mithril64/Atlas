# Contributing to Atlas

## Submission Format

Every `.typ` file submitted to Atlas must have a metadata header, a `---` separator, then the body:

```typst
// id: thm-my-theorem
// type: theorem
// deps: [def-real-numbers]
// tags: [analysis]
---
#statement[
  Let $x in RR$. Then $x^2 >= 0$.
]

#intuition[
  Squaring removes sign.
]

#proof[
  If $x >= 0$, then $x^2 >= 0$ trivially. If $x < 0$, then $-x > 0$, so $x^2 = (-x)^2 >= 0$. $square$
]
```

## Metadata Fields

| Field | Required | Description |
|-------|----------|-------------|
| `// id:` | ✅ | Unique kebab-case identifier. Convention: `{type}-{name}`, e.g. `thm-bolzano-weierstrass`, `def-prime` |
| `// type:` | ✅ | One of: `theorem`, `lemma`, `definition`, `axiom`, `intuition`, `proof` |
| `// deps:` | Optional | Comma-separated list of node IDs this depends on, e.g. `[def-group, ax-choice]` |
| `// tags:` | Optional | Determines which subfolder the file is saved in. First tag = folder name |

## Body Blocks

Use these macros inside the `---` body:

| Macro | Purpose |
|-------|---------|
| `#statement[...]` | The main claim |
| `#intuition[...]` | Informal explanation |
| `#proof[...]` | Formal proof |
| `#corollary[...]` | Derived result |
| `#remark[...]` | Side note |

## Submitting

**Via the IDE** (`ide.html`): Write directly in the browser editor and click **Publish to Atlas**.

**Via the portal** (`submit.html`): Drag and drop a `.typ` file.

Both routes POST to `http://127.0.0.1:3000/api/submit`. The server validates, saves, and opens a GitHub PR. Errors are shown verbatim.

## Local Preview

The IDE (`ide.html`) compiles your Typst live in-browser using a WASM build of the Typst compiler. You can preview and download a PDF before submitting.
