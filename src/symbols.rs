use std::path::Path;

use anyhow::{Context, Result};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor};

use crate::languages::LanguageSupport;

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
    include_public: bool,
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

            if lang.should_skip_node(capture.node, source.as_bytes()) {
                continue;
            }

            if !include_public && lang.is_public(capture.node, source.as_bytes()) {
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
            // Captures named with a leading underscore (e.g. @_m) are query
            // predicates' helpers, not references.
            if query.capture_names()[capture.index as usize].starts_with('_') {
                continue;
            }

            let text = capture
                .node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();

            let Some(name) = lang.normalize_reference(capture.node.kind(), &text) else {
                continue;
            };

            references.push(Reference { name });
        }
    }

    Ok(references)
}
