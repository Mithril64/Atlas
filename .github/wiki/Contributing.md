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
| `// tags:` | Optional | Determines which subfolder the file is saved in. First tag = folder name. Use `demo` to skip the PR pipeline during testing. |

## Body Blocks

Use these macros inside the `---` body:

| Macro | Purpose |
|-------|---------|
| `#statement[...]` | The main claim |
| `#intuition[...]` | Informal explanation |
| `#proof[...]` | Formal proof |
| `#corollary[...]` | Derived result |
| `#remark[...]` | Side note |

All blocks are styled with the Dracula dark theme — previews and downloaded PDFs will both use the dark background.

## Wikilinks

Inside any body block you can reference other nodes using Obsidian-style syntax:

```typst
#proof[
  By [[def-group|the group axioms]] and [[thm-lagrange]]...
]
```

`[[id]]` renders as a clickable link. `[[id|text]]` uses custom display text.

Any node IDs referenced inside `#proof[...]` blocks are **automatically added to `deps`** during compilation — you do not need to list them manually in the header, though doing so is fine.

## Submitting

**Via the IDE** (`ide.html`): Write directly in the browser editor and click **Publish to Atlas**. The IDE compiles a live preview as you type. You can also download the PDF before submitting.

**Via the portal** (`submit.html`): Drag and drop a `.typ` file.

Both routes require you to **log in with GitHub** (button in the top-right of the nav). Your OAuth token is used to open the pull request on your behalf — the PR will appear under your GitHub account.

Submissions with `tags: [demo]` skip the Git/PR step and just return success (useful for local testing without needing GitHub credentials).

## What happens after you submit

1. The server validates the metadata and file format
2. A branch `submission-{id}-{timestamp}` is created and pushed
3. A pull request is opened automatically on GitHub under your account
4. Once the PR is merged, a GitHub Actions workflow deletes the submission branch automatically
5. A maintainer runs `make compile` to regenerate the graph with your new node
