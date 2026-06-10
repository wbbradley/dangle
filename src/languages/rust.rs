use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct RustLanguage;

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
        (attribute_item (attribute (token_tree (string_literal) @ref)))
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
}
