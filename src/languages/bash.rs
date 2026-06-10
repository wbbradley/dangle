use tree_sitter::Language;

use crate::languages::LanguageSupport;

pub struct BashLanguage;

impl LanguageSupport for BashLanguage {
    fn language(&self) -> Language {
        tree_sitter_bash::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        // Both `function f` and `f()` forms have the name: field.
        r#"
        (function_definition name: (word) @def)
        "#
    }

    fn references_query(&self) -> &str {
        // Plain commands are (command name: (command_name (word))) and bare
        // arguments (`trap cleanup EXIT`) are (word) nodes, so one pattern
        // covers both. Quoted arguments are not captured.
        r#"
        (word) @ref
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "main"
    }
}
