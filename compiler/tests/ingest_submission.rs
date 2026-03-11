// compiler/tests/ingest_submission.rs
//
// Integration tests for the `ingest_submission` parser.
// These live outside src/ so they exercise the public(crate) API exactly as
// the server would and don't pollute the main source file.

use math_graph_compiler::ingest_submission;

// ──────────────────────────────────────────────────────────────────────────────
// Helper
// ──────────────────────────────────────────────────────────────────────────────

fn make_doc(id: &str, ty: &str, deps: &str, tags: &str, body: &str) -> String {
    format!("// id: {id}\n// type: {ty}\n// deps: [{deps}]\n// tags: [{tags}]\n---\n{body}")
}

// ──────────────────────────────────────────────────────────────────────────────
// Happy-path: node type variants
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn basic_theorem_parses() {
    let doc = make_doc("thm-foo", "theorem", "", "algebra", "#statement[Foo is true.]");
    let (id, formatted, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(id, "thm-foo");
    assert_eq!(tag, "algebra");
    assert!(
        formatted.starts_with("#theorem("),
        "expected #theorem(…), got: {formatted}"
    );
    assert!(formatted.contains("id:\"thm-foo\""));
    assert!(formatted.contains("#statement[Foo is true.]"));
}

#[test]
fn definition_parses() {
    let doc = make_doc("def-group", "definition", "", "algebra", "#statement[A group is…]");
    let (id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(id, "def-group");
    assert_eq!(tag, "algebra");
}

#[test]
fn axiom_parses() {
    let doc = make_doc("ax-zfc", "axiom", "", "set-theory", "#statement[Given any set…]");
    let (id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(id, "ax-zfc");
    assert_eq!(tag, "set-theory");
}

#[test]
fn lemma_parses() {
    let doc = make_doc("lem-helper", "lemma", "thm-foo", "algebra", "#statement[Helps foo.]");
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(formatted.contains("\"thm-foo\""), "deps should include thm-foo");
}
