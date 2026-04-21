use math_graph_compiler::{
    extract_proof_blocks,
    extract_wikilink_ids,
    fallback_extract_simple,
    render_wikilinks,
};

#[test]
fn wikilinks_render_basic_and_labeled() {
    let base = "https://atlasmath.org";
    let rendered = render_wikilinks("See [[foo]] and [[bar|Baz]]", base);
    assert!(rendered.contains("#link(\"https://atlasmath.org/#foo\")[foo]"));
    assert!(rendered.contains("#link(\"https://atlasmath.org/#bar\")[Baz]"));
}

#[test]
fn wikilink_ids_deduplicated() {
    let text = "[[a]] and [[a|A]] and [[b]]";
    let ids = extract_wikilink_ids(text);
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"a".to_string()));
    assert!(ids.contains(&"b".to_string()));
}

#[test]
fn proof_block_links_extracted() {
    let body = "#statement[hi]\n#proof[Use [[a-lem]] then [[b-lem|B]].]\n#intuition[ok]";
    let blocks = extract_proof_blocks(body);
    assert_eq!(blocks.len(), 1);
    let links = extract_wikilink_ids(&blocks[0]);
    assert_eq!(links.len(), 2);
    assert!(links.contains(&"a-lem".to_string()));
    assert!(links.contains(&"b-lem".to_string()));
}

#[test]
fn fallback_extracts_node_and_body() {
    let src = r#"#theorem(
    id: "thm-demo",
    deps: [def-x, def-y],
    tags: ["algebra", "group"]
)[
#statement[Foo]
#proof[Bar]
]"#;
    let node = fallback_extract_simple(src).expect("should parse");
    assert_eq!(node.id, "thm-demo");
    assert_eq!(node.node_type, "theorem");
    assert!(node.deps.contains(&"def-x".to_string()));
    assert!(node.deps.contains(&"def-y".to_string()));
    assert!(node.tags.contains(&"algebra".to_string()));
    assert!(node.body.contains("Foo"));
    assert!(node.body.contains("Bar"));
}
