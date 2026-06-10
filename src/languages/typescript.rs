use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct TypeScriptLanguage {
    pub tsx: bool,
}

/// Check if a node has an `export_statement` ancestor (i.e. is exported).
pub(crate) fn is_exported(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "export_statement" {
            return true;
        }
        current = parent.parent();
    }
    false
}

/// Test-file conventions shared by TypeScript and JavaScript.
pub(crate) fn is_js_test_file(path: &str) -> bool {
    path.contains(".test.") || path.contains(".spec.") || path.contains("__tests__/")
}

impl LanguageSupport for TypeScriptLanguage {
    fn language(&self) -> Language {
        if self.tsx {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        }
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_declaration name: (identifier) @def)
        (generator_function_declaration name: (identifier) @def)
        (class_declaration name: (type_identifier) @def)
        (abstract_class_declaration name: (type_identifier) @def)
        (interface_declaration name: (type_identifier) @def)
        (type_alias_declaration name: (type_identifier) @def)
        (enum_declaration name: (identifier) @def)
        (program (lexical_declaration (variable_declarator name: (identifier) @def)))
        (program (variable_declaration (variable_declarator name: (identifier) @def)))
        (program (export_statement declaration: (lexical_declaration (variable_declarator name: (identifier) @def))))
        (program (export_statement declaration: (variable_declaration (variable_declarator name: (identifier) @def))))
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        (type_identifier) @ref
        (property_identifier) @ref
        (shorthand_property_identifier) @ref
        (shorthand_property_identifier_pattern) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name.starts_with('_')
    }

    fn is_test_file(&self, path: &str) -> bool {
        // Ambient declaration files (.d.ts) are reference-only
        is_js_test_file(path) || path.ends_with(".d.ts")
    }

    fn is_public(&self, node: Node, _source: &[u8]) -> bool {
        is_exported(node)
    }
}
