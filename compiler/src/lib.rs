// compiler/src/lib.rs
//
// Library entry point.
// Exposes the internal parsing logic publicly so that integration tests
// in `compiler/tests/` can call it without any `#[cfg(test)]` hacks.
//
// The server binary (`main.rs`) imports `ingest_submission` from here via
// `use math_graph_compiler::ingest_submission`.


use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize, Clone)]
pub struct MathNode {
    pub id: String,
    pub node_type: String,
    pub deps: Vec<String>,
    pub tags: Vec<String>,
    pub body: String,
}

// ─── Wikilink Rendering & Parsing Helpers (shared with bin + tests) ─────────

pub fn render_wikilinks(body: &str, base: &str) -> String {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    re.replace_all(body, |caps: &regex::Captures| {
        let id = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let text = caps
            .get(2)
            .map(|m| m.as_str().trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(id);
        format!("#link(\"{}/#{}\")[{}]", base.trim_end_matches('/'), id, text)
    })
    .into_owned()
}

pub fn extract_proof_blocks(body: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut i = 0usize;
    while let Some(rel) = body[i..].find("#proof[") {
        let open = i + rel;
        let mut depth = 0usize;
        let mut start_idx = None;
        let mut end_idx = None;

        for (offset, ch) in body[open..].char_indices() {
            let idx = open + offset;
            match ch {
                '[' => {
                    depth += 1;
                    if depth == 1 {
                        start_idx = Some(idx + 1);
                    }
                }
                ']' => {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = Some(idx);
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        if let (Some(start), Some(end)) = (start_idx, end_idx) {
            blocks.push(body[start..end].to_string());
            i = end + 1;
        } else {
            break;
        }
    }
    blocks
}

pub fn extract_wikilink_ids(text: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    let mut set = HashSet::new();
    for cap in re.captures_iter(text) {
        if let Some(id) = cap.get(1).map(|m| m.as_str().trim()) {
            if !id.is_empty() {
                set.insert(id.to_string());
            }
        }
    }
    set.into_iter().collect()
}

pub fn fallback_extract_simple(src: &str) -> Option<MathNode> {
    let kind_re = Regex::new(r"#(theorem|lemma|definition|axiom|intuition|proof)\s*\(").ok()?;
    let kind_caps = kind_re.captures(src)?;
    let node_type = kind_caps.get(1)?.as_str().to_string();
    let kind_end = kind_caps.get(0)?.end();

    let id = Regex::new("id:\\s*\\\"([^\\\"]+)\\\"").ok()?
        .captures(src)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    let deps = Regex::new(r"deps:\s*\[([^\]]*)\]").ok()?
        .captures(src)
        .map(|c| c.get(1).map(|m| m.as_str()).unwrap_or(""))
        .map(|inner| {
            inner
                .split(',')
                .filter_map(|s| {
                    let t = s.trim().trim_matches('"');
                    if t.is_empty() { None } else { Some(t.to_string()) }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let tags = Regex::new(r"tags:\s*\[([^\]]*)\]").ok()?
        .captures(src)
        .map(|c| c.get(1).map(|m| m.as_str()).unwrap_or(""))
        .map(|inner| {
            inner
                .split(',')
                .filter_map(|s| {
                    let t = s.trim().trim_matches('"');
                    if t.is_empty() { None } else { Some(t.to_string()) }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // find end of argument list so we don't confuse deps/tags arrays for body
    let mut paren_depth = 1usize; // we've seen the opening '(' in the macro
    let mut args_end = None;
    for (offset, ch) in src[kind_end..].char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    args_end = Some(kind_end + offset + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let args_end_idx = args_end.unwrap_or(src.len());

    // crude bracket matching to capture the first content block after the args
    let after_args = &src[args_end_idx..];
    let start_rel = after_args.find('[')?;
    let mut depth = 0usize;
    let mut end_rel = None;
    for (i, ch) in after_args[start_rel..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        end_rel = Some(start_rel + i);
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    let body = end_rel
        .and_then(|end| after_args.get(start_rel + 1..end))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if id.is_empty() || body.is_empty() {
        return None;
    }

    Some(MathNode { id, node_type, deps, tags, body })
}

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
