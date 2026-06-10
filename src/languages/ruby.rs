use tree_sitter::Language;

use crate::languages::LanguageSupport;

pub struct RubyLanguage;

impl LanguageSupport for RubyLanguage {
    fn language(&self) -> Language {
        tree_sitter_ruby::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        // Operator/setter methods (`def ==`, `def foo=`) deliberately not captured.
        r#"
        (method name: (identifier) @def)
        (singleton_method name: (identifier) @def)
        (class name: (constant) @def)
        (class name: (scope_resolution name: (constant) @def))
        (module name: (constant) @def)
        (module name: (scope_resolution name: (constant) @def))
        (assignment left: (constant) @def)
        "#
    }

    fn references_query(&self) -> &str {
        // Symbols (`:foo`, `foo:` hash keys) and string arguments to dynamic
        // dispatch (`send "foo"`, `define_method("foo")`) count as references.
        r#"
        (identifier) @ref
        (constant) @ref
        (simple_symbol) @ref
        (hash_key_symbol) @ref
        (call method: (identifier) @_m
              arguments: (argument_list (string (string_content) @ref))
              (#match? @_m "^(send|public_send|define_method|method|respond_to\\?)$"))
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        matches!(
            name,
            "initialize" | "method_missing" | "respond_to_missing?"
        )
    }

    fn is_test_file(&self, path: &str) -> bool {
        let filename = path.rsplit('/').next().unwrap_or(path);
        filename.ends_with("_spec.rb")
            || filename.ends_with("_test.rb")
            || path.starts_with("spec/")
            || path.starts_with("test/")
            || path.contains("/spec/")
            || path.contains("/test/")
    }

    fn normalize_reference(&self, kind: &str, text: &str) -> Option<String> {
        // Symbol literal text includes the leading colon (`:foo`).
        if kind == "simple_symbol" {
            return Some(text.trim_start_matches(':').to_string());
        }
        Some(text.to_string())
    }
}
