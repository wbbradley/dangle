use tree_sitter::Language;

pub trait LanguageSupport: Send + Sync {
    fn language(&self) -> Language;
    fn definitions_query(&self) -> &str;
    fn references_query(&self) -> &str;
    fn should_ignore(&self, name: &str) -> bool;
}

pub struct RustLanguage;

impl LanguageSupport for RustLanguage {
    fn language(&self) -> Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn definitions_query(&self) -> &str {
        r#"
        (function_item name: (identifier) @def)
        (struct_item name: (type_identifier) @def)
        (enum_item name: (type_identifier) @def)
        (const_item name: (identifier) @def)
        (static_item name: (identifier) @def)
        (mod_item name: (identifier) @def)
        (trait_item name: (type_identifier) @def)
        "#
    }

    fn references_query(&self) -> &str {
        r#"
        (identifier) @ref
        (type_identifier) @ref
        (field_identifier) @ref
        (attribute_item (attribute (token_tree (string_literal) @ref)))
        "#
    }

    fn should_ignore(&self, name: &str) -> bool {
        name == "main" || name.starts_with("test_") || name.starts_with("__")
    }
}

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

pub fn get_language_for_extension(ext: &str) -> Option<&'static dyn LanguageSupport> {
    static RUST: RustLanguage = RustLanguage;
    static PYTHON: PythonLanguage = PythonLanguage;

    match ext {
        "rs" => Some(&RUST),
        "py" => Some(&PYTHON),
        _ => None,
    }
}
