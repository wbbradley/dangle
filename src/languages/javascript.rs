use tree_sitter::{Language, Node};

use crate::languages::{
    LanguageSupport,
    typescript::{is_exported, is_js_test_file},
};

pub struct JavaScriptLanguage;

impl LanguageSupport for JavaScriptLanguage {
    fn language(&self) -> Language {
        tree_sitter_javascript::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_declaration name: (identifier) @def)
        (generator_function_declaration name: (identifier) @def)
        (class_declaration name: (identifier) @def)
        (program (lexical_declaration (variable_declarator name: (identifier) @def)))
        (program (variable_declaration (variable_declarator name: (identifier) @def)))
        (program (export_statement declaration: (lexical_declaration (variable_declarator name: (identifier) @def))))
        (program (export_statement declaration: (variable_declaration (variable_declarator name: (identifier) @def))))
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        (property_identifier) @ref
        (shorthand_property_identifier) @ref
        (shorthand_property_identifier_pattern) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name.starts_with('_')
    }

    fn is_test_file(&self, path: &str) -> bool {
        is_js_test_file(path)
    }

    fn is_public(&self, node: Node, _source: &[u8]) -> bool {
        is_exported(node)
    }
}
