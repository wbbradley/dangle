use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct JavaLanguage;

/// Walk up from a name capture to the enclosing declaration node (the one
/// that owns a `(modifiers)` child), e.g. identifier -> method_declaration,
/// or identifier -> variable_declarator -> field_declaration.
fn enclosing_declaration(node: Node) -> Option<Node> {
    let mut current = node.parent();
    for _ in 0..4 {
        let parent = current?;
        let mut cursor = parent.walk();
        if parent.kind().ends_with("_declaration")
            || parent
                .children(&mut cursor)
                .any(|c| c.kind() == "modifiers")
        {
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

/// The last `.`-segment of an annotation name (e.g. `org.junit.Test` -> `Test`).
fn annotation_simple_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Annotations that mark a definition as externally invoked (test frameworks)
/// or invoked via a supertype (`@Override`).
fn is_skip_annotation(name: &str) -> bool {
    matches!(
        annotation_simple_name(name),
        "Test" | "ParameterizedTest" | "RepeatedTest" | "Override"
    )
}

fn has_skip_annotation(node: Node, source: &[u8]) -> bool {
    let Some(decl) = enclosing_declaration(node) else {
        return false;
    };
    let Some(modifiers) = modifiers_node(decl) else {
        return false;
    };
    let mut cursor = modifiers.walk();
    modifiers.children(&mut cursor).any(|child| {
        matches!(child.kind(), "marker_annotation" | "annotation")
            && child
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .is_some_and(is_skip_annotation)
    })
}

impl LanguageSupport for JavaLanguage {
    fn language(&self) -> Language {
        tree_sitter_java::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        // Constructors deliberately excluded: they are referenced via the class name.
        r#"
        (class_declaration name: (identifier) @def)
        (interface_declaration name: (identifier) @def)
        (enum_declaration name: (identifier) @def)
        (record_declaration name: (identifier) @def)
        (annotation_type_declaration name: (identifier) @def)
        (method_declaration name: (identifier) @def)
        (field_declaration declarator: (variable_declarator name: (identifier) @def))
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        (type_identifier) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        matches!(name, "main" | "toString" | "equals" | "hashCode")
    }

    fn should_skip_node(&self, node: Node, source: &[u8]) -> bool {
        has_skip_annotation(node, source)
    }

    fn is_public(&self, node: Node, source: &[u8]) -> bool {
        let Some(decl) = enclosing_declaration(node) else {
            return false;
        };
        let Some(modifiers) = modifiers_node(decl) else {
            return false;
        };
        let mut cursor = modifiers.walk();
        modifiers
            .children(&mut cursor)
            .any(|c| c.utf8_text(source) == Ok("public"))
    }

    fn is_test_file(&self, path: &str) -> bool {
        let filename = path.rsplit('/').next().unwrap_or(path);
        path.contains("/src/test/")
            || filename.ends_with("Test.java")
            || (filename.starts_with("Test") && filename.ends_with(".java"))
    }
}
