use tree_sitter::{Language, Node};

use crate::languages::LanguageSupport;

pub struct LuaLanguage;

impl LanguageSupport for LuaLanguage {
    fn language(&self) -> Language {
        tree_sitter_lua::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_declaration name: (identifier) @def)
        (function_declaration name: (dot_index_expression field: (identifier) @def))
        (function_declaration name: (method_index_expression method: (identifier) @def))
        (variable_declaration (assignment_statement (variable_list name: (identifier) @def)))
        (assignment_statement
          (variable_list name: (dot_index_expression field: (identifier) @def))
          (expression_list (function_definition)))
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        // Convention for intentionally-unused locals, plus metamethods like __index.
        name.starts_with('_')
    }

    fn is_public(&self, node: Node, _source: &[u8]) -> bool {
        // Module-field style (`function M.foo()`, `function M:bar()`,
        // `M.foo = function()`) is Lua's "export" idiom.
        node.parent()
            .is_some_and(|p| matches!(p.kind(), "dot_index_expression" | "method_index_expression"))
    }

    fn is_test_file(&self, path: &str) -> bool {
        let filename = path.rsplit('/').next().unwrap_or(path);
        path.contains("test_")
            || path.contains("/tests/")
            || path.starts_with("tests/")
            || filename.ends_with("_spec.lua")
            || path.contains("/spec/")
            || path.starts_with("spec/")
    }
}
