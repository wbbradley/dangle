use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct RustLanguage;

/// Check if a string is a valid Rust identifier (alphanumeric + underscore, not starting with digit)
fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Extract inline format-arg identifiers from a macro string literal's raw text:
/// `"query={QUERY}"` → ["QUERY"], `"{x:>w$}"` → ["x", "w"]. Positional (`{}`, `{0}`)
/// and escaped (`{{name}}`) groups yield nothing. Quote/raw-string delimiters need
/// no special handling since only brace groups are inspected.
fn extract_format_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        if rest[open + 1..].starts_with('{') {
            // Escaped `{{`
            rest = &rest[open + 2..];
            continue;
        }
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        let group = &after[..close];
        let (arg, spec) = match group.find(':') {
            Some(i) => (&group[..i], &group[i + 1..]),
            None => (group, ""),
        };
        if is_valid_identifier(arg) {
            out.push(arg.to_string());
        }
        // Width/precision args in the spec reference identifiers suffixed with `$`
        // (e.g. `{:>w$}`, `{:.p$}`).
        let mut ident_start = None;
        for (i, c) in spec.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                ident_start.get_or_insert(i);
            } else {
                if c == '$'
                    && let Some(start) = ident_start
                {
                    let id = &spec[start..i];
                    if is_valid_identifier(id) {
                        out.push(id.to_string());
                    }
                }
                ident_start = None;
            }
        }
        rest = &after[close + 1..];
    }
    out
}

/// Check if a node is inside a Rust trait impl block (as opposed to an inherent impl)
fn is_inside_trait_impl(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "impl_item" {
            // A trait impl has a child with field name "trait"
            return parent.child_by_field_name("trait").is_some();
        }
        current = parent.parent();
    }
    false
}

/// Check if a node is a public trait (has visibility modifier)
fn is_public_trait(node: Node) -> bool {
    if let Some(parent) = node.parent()
        && parent.kind() == "trait_item"
    {
        // Check if the trait has a visibility_modifier child
        let mut cursor = parent.walk();
        for child in parent.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                return true;
            }
        }
    }
    false
}

/// Check if an attribute indicates the definition should be skipped
fn is_skip_attribute(attr_text: &str) -> bool {
    // Skip #[test] functions
    if attr_text.contains("test") {
        return true;
    }
    // Skip #[allow(unused)], #[allow(dead_code)], etc.
    if attr_text.contains("allow")
        && (attr_text.contains("unused") || attr_text.contains("dead_code"))
    {
        return true;
    }
    false
}

/// Check if a definition has an attribute that indicates it should be skipped
fn has_skip_attribute(node: Node, source: &[u8]) -> bool {
    if let Some(parent) = node.parent() {
        // Check for attribute_item children of the parent
        let mut cursor = parent.walk();
        for child in parent.children(&mut cursor) {
            if child.kind() == "attribute_item"
                && let Ok(text) = child.utf8_text(source)
                && is_skip_attribute(text)
            {
                return true;
            }
        }
        // Also check preceding siblings (attributes may be separate nodes)
        let mut sibling = parent.prev_sibling();
        while let Some(sib) = sibling {
            if sib.kind() == "attribute_item" {
                if let Ok(text) = sib.utf8_text(source)
                    && is_skip_attribute(text)
                {
                    return true;
                }
            } else if sib.kind() != "line_comment" && sib.kind() != "block_comment" {
                // Stop if we hit something that's not an attribute or comment
                break;
            }
            sibling = sib.prev_sibling();
        }
    }
    false
}

impl LanguageSupport for RustLanguage {
    fn language(&self) -> Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_item name: (identifier) @def)
        (struct_item name: (type_identifier) @def)
        (enum_item name: (type_identifier) @def)
        (const_item name: (identifier) @def)
        (static_item name: (identifier) @def)
        (mod_item name: (identifier) @def)
        (trait_item name: (type_identifier) @def)
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        (type_identifier) @ref
        (field_identifier) @ref
        (attribute_item (attribute (token_tree (string_literal) @attr_string)))
        (token_tree [(string_literal) (raw_string_literal)] @macro_string)
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "main" || name.starts_with("test_") || name.starts_with("__")
    }

    fn should_skip_node(&self, node: Node, source: &[u8]) -> bool {
        // Skip functions inside trait impls (they're called indirectly via the trait),
        // and definitions with #[test], #[allow(unused)], #[allow(dead_code)], etc.
        is_inside_trait_impl(node) || has_skip_attribute(node, source)
    }

    fn is_public(&self, node: Node, _source: &[u8]) -> bool {
        // Rust public policy: only public traits (they may be implemented downstream)
        is_public_trait(node)
    }

    fn normalize_reference(&self, capture: &str, text: &str) -> Vec<String> {
        match capture {
            // String literals in attributes (e.g., #[serde(default = "module::func_name")]):
            // strip quotes and take the last path segment.
            "attr_string" => {
                let inner = text.trim_matches('"');
                let leaf = inner.rsplit("::").next().unwrap_or(inner);
                is_valid_identifier(leaf)
                    .then(|| leaf.to_string())
                    .into_iter()
                    .collect()
            }
            // String literals in macro token trees: extract `{ident}` inline format args.
            "macro_string" => extract_format_args(text),
            _ => vec![text.to_string()],
        }
    }
}
