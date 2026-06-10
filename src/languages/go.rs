use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct GoLanguage;

impl LanguageSupport for GoLanguage {
    fn language(&self) -> Language {
        tree_sitter_go::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_declaration name: (identifier) @def)
        (method_declaration name: (field_identifier) @def)
        (source_file (type_declaration (type_spec name: (type_identifier) @def)))
        (source_file (type_declaration (type_alias name: (type_identifier) @def)))
        (source_file (const_declaration (const_spec name: (identifier) @def)))
        (source_file (var_declaration (var_spec name: (identifier) @def)))
        (source_file (var_declaration (var_spec_list (var_spec name: (identifier) @def))))
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        (type_identifier) @ref
        (field_identifier) @ref
        (package_identifier) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "main"
            || name == "init"
            || name == "_"
            || name.starts_with("Test")
            || name.starts_with("Benchmark")
            || name.starts_with("Fuzz")
            || name.starts_with("Example")
    }

    fn is_public(&self, node: Node, source: &[u8]) -> bool {
        // Go exports by capitalization
        node.utf8_text(source)
            .is_ok_and(|name| name.chars().next().is_some_and(char::is_uppercase))
    }

    fn is_test_file(&self, path: &str) -> bool {
        path.ends_with("_test.go")
    }
}
