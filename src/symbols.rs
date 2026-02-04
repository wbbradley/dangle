use std::path::Path;

use anyhow::{Context, Result};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor};

use crate::languages::LanguageSupport;

/// Check if a string is a valid Rust identifier (alphanumeric + underscore, not starting with digit)
fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
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

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: String,
    pub has_nodangle: bool,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub name: String,
}

pub fn extract_definitions(
    path: &Path,
    source: &str,
    lang: &dyn LanguageSupport,
) -> Result<Vec<Definition>> {
    let mut parser = Parser::new();
    parser
        .set_language(&lang.language())
        .context("Failed to set language")?;

    let tree = parser
        .parse(source, None)
        .context("Failed to parse source")?;

    let query =
        Query::new(&lang.language(), lang.definitions_query()).context("Invalid query syntax")?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let lines: Vec<&str> = source.lines().collect();
    let file_str = path.to_string_lossy().to_string();

    let mut definitions = Vec::new();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let name = capture
                .node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();

            if lang.should_ignore(&name) {
                continue;
            }

            // Skip functions inside trait impls (they're called indirectly via the trait)
            if is_inside_trait_impl(capture.node) {
                continue;
            }

            // Skip public traits (they may be implemented by downstream crates)
            if is_public_trait(capture.node) {
                continue;
            }

            // Skip definitions with #[test], #[allow(unused)], #[allow(dead_code)], etc.
            if has_skip_attribute(capture.node, source.as_bytes()) {
                continue;
            }

            let pos = capture.node.start_position();
            let line = pos.row;
            let column = pos.column;
            let line_text = lines.get(line).unwrap_or(&"");

            let has_nodangle = line_text.contains("nodangle") || line_text.contains("# nodangle");

            let kind = capture
                .node
                .parent()
                .map_or("unknown", |p: Node<'_>| p.kind());

            definitions.push(Definition {
                name,
                file: file_str.clone(),
                line: line + 1,
                column: column + 1,
                kind: kind.to_string(),
                has_nodangle,
            });
        }
    }

    Ok(definitions)
}

pub fn extract_references(source: &str, lang: &dyn LanguageSupport) -> Result<Vec<Reference>> {
    let mut parser = Parser::new();
    parser
        .set_language(&lang.language())
        .context("Failed to set language")?;

    let tree = parser
        .parse(source, None)
        .context("Failed to parse source")?;

    let query =
        Query::new(&lang.language(), lang.references_query()).context("Invalid query syntax")?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut references = Vec::new();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let text = capture
                .node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();

            // Handle string literals in attributes (e.g., #[serde(default = "func_name")])
            let name = if capture.node.kind() == "string_literal" {
                // Strip surrounding quotes
                let inner = text.trim_matches('"');
                // Handle paths like "module::func" by taking the last segment
                let leaf = inner.rsplit("::").next().unwrap_or(inner);
                if is_valid_identifier(leaf) {
                    leaf.to_string()
                } else {
                    continue;
                }
            } else {
                text
            };

            references.push(Reference { name });
        }
    }

    Ok(references)
}
