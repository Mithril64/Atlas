// compiler/src/lib.rs
//
// Library entry point.
// Exposes the internal parsing logic publicly so that integration tests
// in `compiler/tests/` can call it without any `#[cfg(test)]` hacks.
//
// The server binary (`main.rs`) imports `ingest_submission` from here via
// `use math_graph_compiler::ingest_submission`.


use regex::Regex;

pub fn ingest_submission(raw_text: &str) -> Result<(String, String, String), String> {
    let re_id   = Regex::new(r"//\s*id:\s*([^\n\r]+)").unwrap();
    let re_type = Regex::new(r"//\s*type:\s*([^\n\r]+)").unwrap();
    let re_deps = Regex::new(r"//\s*deps:\s*\[([^\]]*)\]").unwrap();
    let re_tags = Regex::new(r"//\s*tags:\s*\[([^\]]*)\]").unwrap();

    let id = re_id
        .captures(raw_text).ok_or("No id")?
        .get(1).map(|m| m.as_str().trim()).ok_or("Malformed id")?
        .to_string();

    let node_type = re_type
        .captures(raw_text).ok_or("No type")?
        .get(1).map(|m| m.as_str().trim()).ok_or("Malformed type")?
        .to_string();

    let deps_raw = re_deps
        .captures(raw_text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim())
        .unwrap_or("");

    let tags_raw = re_tags
        .captures(raw_text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim())
        .unwrap_or("");

    let mut primary_tag = "uncategorized".to_string();
    if !tags_raw.is_empty() {
        if let Some(first) = tags_raw.split(',').next() {
            primary_tag = first
                .trim()
                .trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']')
                .to_lowercase()
                .replace(" ", "-");
        }
    }

    let body = if let Some(pos) = raw_text.find("---") {
        raw_text[pos + 3..].trim().to_string()
    } else {
        raw_text.trim().to_string()
    };

    let format_arr = |raw: &str| {
        if raw.is_empty() {
            return "()".to_string();
        }
        let items: Vec<String> = raw
            .split(',')
            .map(|s| {
                format!(
                    "\"{}\"",
                    s.trim().trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']')
                )
            })
            .collect();
        format!("({},)", items.join(", "))
    };

    let formatted = format!(
        "#{}(id:\"{}\",deps:{},tags:{})[\n{}\n]",
        node_type,
        id,
        format_arr(deps_raw),
        format_arr(tags_raw),
        body
    );
    Ok((id, formatted, primary_tag))
}
