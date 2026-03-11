// compiler/tests/errors.rs
//
// Integration tests that verify `ingest_submission` rejects invalid input
// with appropriate errors.

use math_graph_compiler::ingest_submission;

fn make_doc(id: &str, ty: &str, deps: &str, tags: &str, body: &str) -> String {
    format!("// id: {id}\n// type: {ty}\n// deps: [{deps}]\n// tags: [{tags}]\n---\n{body}")
}

// ──────────────────────────────────────────────────────────────────────────────
// Required-field rejections
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn missing_id_returns_err() {
    let doc = "// type: theorem\n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
    assert!(ingest_submission(doc).is_err(), "should fail when id is absent");
}

#[test]
fn missing_type_returns_err() {
    let doc = "// id: thm-notype\n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
    assert!(ingest_submission(doc).is_err(), "should fail when type is absent");
}

#[test]
fn completely_empty_input_returns_err() {
    assert!(ingest_submission("").is_err());
}

#[test]
fn whitespace_only_input_returns_err() {
    assert!(ingest_submission("   \n  \t  \n").is_err());
}

#[test]
fn unrelated_text_returns_err() {
    assert!(ingest_submission("Hello, world!").is_err());
}

#[test]
fn only_separator_returns_err() {
    assert!(ingest_submission("---").is_err());
}

// ──────────────────────────────────────────────────────────────────────────────
// Malformed but close input
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn id_present_but_type_missing_at_eof_returns_err() {
    // id present but no type line at all
    let doc = "// id: thm-only-id\n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
    assert!(ingest_submission(doc).is_err());
}

#[test]
fn all_fields_present_is_ok() {
    // Sanity: the complement of the above must succeed
    let doc = make_doc("thm-ok", "theorem", "", "algebra", "#statement[.]");
    assert!(ingest_submission(&doc).is_ok());
}
