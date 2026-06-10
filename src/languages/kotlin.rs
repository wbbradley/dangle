use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct KotlinLanguage;

/// Walk up from a name capture to the enclosing declaration node, e.g.
/// identifier -> function_declaration, or
/// identifier -> variable_declaration -> property_declaration.
fn enclosing_declaration(node: Node) -> Option<Node> {
    let mut current = node.parent();
    for _ in 0..4 {
        let parent = current?;
        if matches!(
            parent.kind(),
            "function_declaration"
                | "class_declaration"
                | "object_declaration"
                | "property_declaration"
                | "type_alias"
        ) {
            return Some(parent);
        }
        current = parent.parent();
    }
    None
}

/// Find the `(modifiers)` child of a declaration node.
fn modifiers_node(decl: Node) -> Option<Node> {
    let mut cursor = decl.walk();
    decl.children(&mut cursor).find(|c| c.kind() == "modifiers")
}

/// Annotations that mark a definition as externally invoked (test frameworks),
/// plus the `override` modifier (overrides are invoked via the supertype).
fn has_skip_modifier(node: Node, source: &[u8]) -> bool {
    let Some(decl) = enclosing_declaration(node) else {
        return false;
    };
    let Some(modifiers) = modifiers_node(decl) else {
        return false;
    };
    let mut cursor = modifiers.walk();
    modifiers.children(&mut cursor).any(|child| {
        let text = child.utf8_text(source).unwrap_or("");
        (child.kind() == "annotation" && text.contains("Test")) || text == "override"
    })
}

impl LanguageSupport for KotlinLanguage {
    fn language(&self) -> Language {
        tree_sitter_kotlin_ng::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_declaration name: (identifier) @def)
        (class_declaration name: (identifier) @def)
        (object_declaration name: (identifier) @def)
        (property_declaration (variable_declaration (identifier) @def))
        (type_alias type: (identifier) @def)
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "main"
    }

    fn should_skip_node(&self, node: Node, source: &[u8]) -> bool {
        has_skip_modifier(node, source)
    }

    fn is_public(&self, node: Node, source: &[u8]) -> bool {
        let Some(decl) = enclosing_declaration(node) else {
            return false;
        };
        // Only declarations in a declaration context default to public;
        // function-local `val`/`var` must remain reportable.
        if !decl
            .parent()
            .is_some_and(|p| matches!(p.kind(), "source_file" | "class_body" | "enum_class_body"))
        {
            return false;
        }
        // Kotlin default visibility is public unless marked private/internal.
        let Some(modifiers) = modifiers_node(decl) else {
            return true;
        };
        let mut cursor = modifiers.walk();
        !modifiers.children(&mut cursor).any(|c| {
            c.kind() == "visibility_modifier"
                && matches!(c.utf8_text(source), Ok("private") | Ok("internal"))
        })
    }

    fn is_test_file(&self, path: &str) -> bool {
        let filename = path.rsplit('/').next().unwrap_or(path);
        path.contains("/src/test/")
            || path.starts_with("src/test/")
            || filename.ends_with("Test.kt")
            || path.contains("test_")
            || path.contains("/tests/")
    }
}
