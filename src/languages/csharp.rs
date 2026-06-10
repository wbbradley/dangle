use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct CSharpLanguage;

/// The last `.`-segment of an attribute name (e.g. `Xunit.Fact` -> `Fact`).
fn attribute_simple_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Attributes that mark a method as invoked by a test framework.
fn is_skip_attribute(name: &str) -> bool {
    matches!(
        attribute_simple_name(name),
        "Test" | "Fact" | "Theory" | "TestMethod"
    )
}

fn has_skip_attribute(node: Node, source: &[u8]) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "method_declaration" {
        return false;
    }
    let mut cursor = parent.walk();
    parent.children(&mut cursor).any(|child| {
        if child.kind() != "attribute_list" {
            return false;
        }
        let mut attr_cursor = child.walk();
        child.children(&mut attr_cursor).any(|attr| {
            attr.kind() == "attribute"
                && attr
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source).ok())
                    .is_some_and(is_skip_attribute)
        })
    })
}

impl LanguageSupport for CSharpLanguage {
    fn language(&self) -> Language {
        tree_sitter_c_sharp::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (class_declaration name: (identifier) @def)
        (interface_declaration name: (identifier) @def)
        (struct_declaration name: (identifier) @def)
        (enum_declaration name: (identifier) @def)
        (record_declaration name: (identifier) @def)
        (method_declaration name: (identifier) @def)
        (property_declaration name: (identifier) @def)
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "Main"
    }

    fn should_skip_node(&self, node: Node, source: &[u8]) -> bool {
        has_skip_attribute(node, source)
    }

    fn is_public(&self, node: Node, source: &[u8]) -> bool {
        let Some(parent) = node.parent() else {
            return false;
        };
        let mut cursor = parent.walk();
        parent
            .children(&mut cursor)
            .any(|c| c.kind() == "modifier" && c.utf8_text(source) == Ok("public"))
    }

    fn is_test_file(&self, path: &str) -> bool {
        path.ends_with("Test.cs") || path.ends_with("Tests.cs") || path.contains("/tests/")
    }
}
