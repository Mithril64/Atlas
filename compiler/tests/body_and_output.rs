// compiler/tests/body_and_output.rs
//
// Integration tests for body extraction and the shape of the formatted Typst
// output produced by `ingest_submission`.

use math_graph_compiler::ingest_submission;

fn make_doc(id: &str, ty: &str, deps: &str, tags: &str, body: &str) -> String {
    format!("// id: {id}\n// type: {ty}\n// deps: [{deps}]\n// tags: [{tags}]\n---\n{body}")
}

// ──────────────────────────────────────────────────────────────────────────────
// Body extraction
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn body_extracted_after_separator() {
    let doc = make_doc("thm-body", "theorem", "", "algebra", "#statement[The body lives here.]");
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(
        formatted.contains("The body lives here."),
        "body content should survive"
    );
}

#[test]
fn no_separator_uses_whole_text_as_body() {
    // No `---` separator — body is the entire trimmed text
    let doc =
        "// id: thm-nosep\n// type: theorem\n// deps: []\n// tags: [algebra]\n#statement[No sep.]";
    let (_id, formatted, _tag) = ingest_submission(doc).unwrap();
    assert!(formatted.contains("#statement[No sep.]"));
}

#[test]
fn multiline_body_preserved() {
    let body = "#statement[\n  Line one.\n  Line two.\n]\n\n#intuition[\n  Some intuition.\n]";
    let doc = make_doc("thm-multi-line", "theorem", "", "algebra", body);
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(formatted.contains("Line one."));
    assert!(formatted.contains("Line two."));
    assert!(formatted.contains("Some intuition."));
}

#[test]
fn body_with_math_dollars_preserved() {
    let body = "#statement[Let $x \\in \\mathbb{R}$. Then $x^2 \\ge 0$.]";
    let doc = make_doc("thm-math", "theorem", "", "analysis", body);
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(formatted.contains("$x^2 \\ge 0$"));
}

// ──────────────────────────────────────────────────────────────────────────────
// Output structure
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn formatted_output_is_valid_typst_call() {
    let doc = make_doc("thm-fmt", "theorem", "dep-a", "algebra", "#statement[Valid.]");
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    // Must look like: #theorem(id:"...",deps:(...),tags:(...))[…body…]
    assert!(formatted.starts_with("#theorem("));
    assert!(formatted.contains("id:\"thm-fmt\""));
    assert!(formatted.ends_with(']'));
}

#[test]
fn formatted_output_contains_tags_tuple() {
    let doc = make_doc("thm-tup", "theorem", "", "analysis", "#statement[Tup.]");
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    // tags should be formatted as ("analysis",)
    assert!(formatted.contains("tags:("), "tags should be a tuple");
}

#[test]
fn whitespace_around_id_trimmed() {
    let doc =
        "// id:   thm-spaces   \n// type: theorem\n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
    let (id, _fmt, _tag) = ingest_submission(doc).unwrap();
    assert_eq!(id, "thm-spaces");
}

#[test]
fn whitespace_around_type_trimmed() {
    let doc =
        "// id: thm-typespace\n// type:   theorem   \n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
    let (_id, formatted, _tag) = ingest_submission(doc).unwrap();
    assert!(formatted.starts_with("#theorem("));
}
