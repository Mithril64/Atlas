// compiler/tests/deps_and_tags.rs
//
// Integration tests focused on the deps/tags field parsing logic
// inside `ingest_submission`.

use math_graph_compiler::ingest_submission;

fn make_doc(id: &str, ty: &str, deps: &str, tags: &str, body: &str) -> String {
    format!("// id: {id}\n// type: {ty}\n// deps: [{deps}]\n// tags: [{tags}]\n---\n{body}")
}

// ──────────────────────────────────────────────────────────────────────────────
// Dependencies
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn empty_deps_gives_unit_tuple() {
    let doc = make_doc("thm-nodep", "theorem", "", "algebra", "#statement[No deps.]");
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(formatted.contains("deps:()"), "empty deps should be ()");
}

#[test]
fn single_dep_preserved() {
    let doc = make_doc("thm-one-dep", "theorem", "def-a", "algebra", "#statement[One dep.]");
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(formatted.contains("\"def-a\""));
}

#[test]
fn multiple_deps_preserved() {
    let doc = make_doc(
        "thm-bar",
        "theorem",
        "def-a, def-b, def-c",
        "analysis",
        "#statement[Bar.]",
    );
    let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
    assert!(formatted.contains("\"def-a\""));
    assert!(formatted.contains("\"def-b\""));
    assert!(formatted.contains("\"def-c\""));
}

// ──────────────────────────────────────────────────────────────────────────────
// Tags → folder name
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn single_tag_becomes_folder() {
    let doc = make_doc("thm-t1", "theorem", "", "algebra", "#statement[T1.]");
    let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(tag, "algebra");
}

#[test]
fn multiple_tags_first_becomes_folder() {
    let doc = make_doc("thm-multi", "theorem", "", "topology, analysis", "#statement[Multi.]");
    let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(tag, "topology", "first tag should be the folder");
}

#[test]
fn tag_is_lowercased_and_spaces_become_dashes() {
    let doc = make_doc("thm-t", "theorem", "", "Linear Algebra", "#statement[T.]");
    let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(tag, "linear-algebra");
}

#[test]
fn quoted_tags_stripped() {
    let doc = make_doc("thm-q", "theorem", "", "\"algebra\"", "#statement[Q.]");
    let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(tag, "algebra");
}

#[test]
fn single_quoted_tags_stripped() {
    let doc = make_doc("thm-sq", "theorem", "", "'topology'", "#statement[SQ.]");
    let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(tag, "topology");
}

#[test]
fn empty_tags_gives_uncategorized() {
    let doc = "// id: thm-x\n// type: theorem\n// deps: []\n// tags: []\n---\n#statement[X.]";
    let (_id, _fmt, tag) = ingest_submission(doc).unwrap();
    assert_eq!(tag, "uncategorized");
}

#[test]
fn demo_tag_preserved() {
    let doc = make_doc("thm-demo-1", "theorem", "", "demo", "#statement[Demo.]");
    let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
    assert_eq!(tag, "demo");
}
