use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct PhpLanguage;

/// Walk up from a name capture to the enclosing declaration node (e.g.
/// name -> method_declaration, or name -> const_element -> const_declaration).
fn enclosing_declaration(node: Node) -> Option<Node> {
    let mut current = node.parent();
    for _ in 0..4 {
        let parent = current?;
        if parent.kind().ends_with("_declaration") {
            return Some(parent);
        }
        current = parent.parent();
    }
    None
}

impl LanguageSupport for PhpLanguage {
    fn language(&self) -> Language {
        tree_sitter_php::LANGUAGE_PHP.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_definition name: (name) @def)
        (class_declaration name: (name) @def)
        (interface_declaration name: (name) @def)
        (trait_declaration name: (name) @def)
        (enum_declaration name: (name) @def)
        (enum_case name: (name) @def)
        (method_declaration name: (name) @def)
        (const_declaration (const_element (name) @def))
        "#
    }

    fn references_query(&self) -> &str {
        // (name) covers calls, static/member access, type hints, `new`, and
        // `use` imports.
        r#"
        (name) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        // Magic methods: __construct, __get, __toString, ...
        name.starts_with("__")
    }

    fn is_public(&self, node: Node, source: &[u8]) -> bool {
        // Only explicit `public` members are skipped; no-modifier members and
        // all top-level functions/classes stay reportable.
        let Some(decl) = enclosing_declaration(node) else {
            return false;
        };
        let mut cursor = decl.walk();
        decl.children(&mut cursor).any(|c| {
            c.kind() == "visibility_modifier"
                && c.utf8_text(source).is_ok_and(|t| t.starts_with("public"))
        })
    }

    fn is_test_file(&self, path: &str) -> bool {
        let filename = path.rsplit('/').next().unwrap_or(path);
        filename.ends_with("Test.php") || path.starts_with("tests/") || path.contains("/tests/")
    }
}
