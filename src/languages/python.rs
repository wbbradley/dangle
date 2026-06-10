use tree_sitter::Language;

use crate::languages::LanguageSupport;

pub struct PythonLanguage;

impl LanguageSupport for PythonLanguage {
    fn language(&self) -> Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_definition name: (identifier) @def)
        (class_definition name: (identifier) @def)
        (module (expression_statement (assignment left: (identifier) @def)))
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "main"
            || name.starts_with("test_")
            || name.starts_with("Test")
            || name.starts_with("__")
    }
}
