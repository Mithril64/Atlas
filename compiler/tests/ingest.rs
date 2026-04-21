use math_graph_compiler::ingest_submission;

fn make_doc(id: &str, ty: &str, deps: &str, tags: &str, body: &str) -> String {
    format!("// id: {id}\n// type: {ty}\n// deps: [{deps}]\n// tags: [{tags}]\n---\n{body}")
}

mod happy_path {
    use super::*;

    #[test]
    fn theorem_parses() {
        let doc = make_doc("thm-foo", "theorem", "", "algebra", "#statement[Foo is true.]");
        let (id, formatted, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(id, "thm-foo");
        assert_eq!(tag, "algebra");
        assert!(formatted.starts_with("#theorem("), "got: {formatted}");
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
        assert!(formatted.contains("\"thm-foo\""));
    }
}

mod deps {
    use super::*;

    #[test]
    fn empty_gives_unit_tuple() {
        let doc = make_doc("thm-nodep", "theorem", "", "algebra", "#statement[No deps.]");
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("deps:()"));
    }

    #[test]
    fn single_preserved() {
        let doc = make_doc("thm-one-dep", "theorem", "def-a", "algebra", "#statement[One.]");
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("\"def-a\""));
    }

    #[test]
    fn multiple_preserved() {
        let doc = make_doc("thm-bar", "theorem", "def-a, def-b, def-c", "analysis", "#statement[Bar.]");
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("\"def-a\""));
        assert!(formatted.contains("\"def-b\""));
        assert!(formatted.contains("\"def-c\""));
    }
}

mod tags {
    use super::*;

    #[test]
    fn single_becomes_folder() {
        let doc = make_doc("thm-t1", "theorem", "", "algebra", "#statement[T1.]");
        let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(tag, "algebra");
    }

    #[test]
    fn first_becomes_folder() {
        let doc = make_doc("thm-multi", "theorem", "", "topology, analysis", "#statement[Multi.]");
        let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(tag, "topology");
    }

    #[test]
    fn lowercased_and_hyphenated() {
        let doc = make_doc("thm-t", "theorem", "", "Linear Algebra", "#statement[T.]");
        let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(tag, "linear-algebra");
    }

    #[test]
    fn double_quotes_stripped() {
        let doc = make_doc("thm-q", "theorem", "", "\"algebra\"", "#statement[Q.]");
        let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(tag, "algebra");
    }

    #[test]
    fn single_quotes_stripped() {
        let doc = make_doc("thm-sq", "theorem", "", "'topology'", "#statement[SQ.]");
        let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(tag, "topology");
    }

    #[test]
    fn empty_gives_uncategorized() {
        let doc = "// id: thm-x\n// type: theorem\n// deps: []\n// tags: []\n---\n#statement[X.]";
        let (_id, _fmt, tag) = ingest_submission(doc).unwrap();
        assert_eq!(tag, "uncategorized");
    }

    #[test]
    fn demo_preserved() {
        let doc = make_doc("thm-demo-1", "theorem", "", "demo", "#statement[Demo.]");
        let (_id, _fmt, tag) = ingest_submission(&doc).unwrap();
        assert_eq!(tag, "demo");
    }
}

mod body {
    use super::*;

    #[test]
    fn extracted_after_separator() {
        let doc = make_doc("thm-body", "theorem", "", "algebra", "#statement[The body lives here.]");
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("The body lives here."));
    }

    #[test]
    fn no_separator_uses_whole_text() {
        let doc = "// id: thm-nosep\n// type: theorem\n// deps: []\n// tags: [algebra]\n#statement[No sep.]";
        let (_id, formatted, _tag) = ingest_submission(doc).unwrap();
        assert!(formatted.contains("#statement[No sep.]"));
    }

    #[test]
    fn multiline_preserved() {
        let body = "#statement[\n  Line one.\n  Line two.\n]\n\n#intuition[\n  Some intuition.\n]";
        let doc = make_doc("thm-multi-line", "theorem", "", "algebra", body);
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("Line one."));
        assert!(formatted.contains("Line two."));
        assert!(formatted.contains("Some intuition."));
    }

    #[test]
    fn math_dollars_preserved() {
        let body = "#statement[Let $x \\in \\mathbb{R}$. Then $x^2 \\ge 0$.]";
        let doc = make_doc("thm-math", "theorem", "", "analysis", body);
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("$x^2 \\ge 0$"));
    }
}

mod output {
    use super::*;

    #[test]
    fn valid_typst_call_shape() {
        let doc = make_doc("thm-fmt", "theorem", "dep-a", "algebra", "#statement[Valid.]");
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.starts_with("#theorem("));
        assert!(formatted.contains("id:\"thm-fmt\""));
        assert!(formatted.ends_with(']'));
    }

    #[test]
    fn tags_formatted_as_tuple() {
        let doc = make_doc("thm-tup", "theorem", "", "analysis", "#statement[Tup.]");
        let (_id, formatted, _tag) = ingest_submission(&doc).unwrap();
        assert!(formatted.contains("tags:("));
    }

    #[test]
    fn whitespace_around_id_trimmed() {
        let doc = "// id:   thm-spaces   \n// type: theorem\n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
        let (id, _fmt, _tag) = ingest_submission(doc).unwrap();
        assert_eq!(id, "thm-spaces");
    }

    #[test]
    fn whitespace_around_type_trimmed() {
        let doc = "// id: thm-typespace\n// type:   theorem   \n// deps: []\n// tags: [algebra]\n---\n#statement[.]";
        let (_id, formatted, _tag) = ingest_submission(doc).unwrap();
        assert!(formatted.starts_with("#theorem("));
    }
}

mod errors {
    use super::*;

    #[test]
    fn missing_id() {
        assert!(ingest_submission("// type: theorem\n// deps: []\n// tags: [algebra]\n---\n#statement[.]").is_err());
    }

    #[test]
    fn missing_type() {
        assert!(ingest_submission("// id: thm-notype\n// deps: []\n// tags: [algebra]\n---\n#statement[.]").is_err());
    }

    #[test]
    fn empty_input() {
        assert!(ingest_submission("").is_err());
    }

    #[test]
    fn whitespace_only() {
        assert!(ingest_submission("   \n  \t  \n").is_err());
    }

    #[test]
    fn unrelated_text() {
        assert!(ingest_submission("Hello, world!").is_err());
    }

    #[test]
    fn only_separator() {
        assert!(ingest_submission("---").is_err());
    }

    #[test]
    fn id_without_type() {
        assert!(ingest_submission("// id: thm-only-id\n// deps: []\n// tags: [algebra]\n---\n#statement[.]").is_err());
    }

    #[test]
    fn all_fields_present_is_ok() {
        let doc = make_doc("thm-ok", "theorem", "", "algebra", "#statement[.]");
        assert!(ingest_submission(&doc).is_ok());
    }
}
