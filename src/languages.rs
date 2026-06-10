pub mod csharp;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

use tree_sitter::{Language, Node};

pub trait LanguageSupport: Send + Sync {
    fn language(&self) -> Language;
    fn definitions_query(&self) -> &str;
    fn references_query(&self) -> &str;
    fn should_ignore(&self, name: &str) -> bool;

    /// Files whose definitions should not be reported (references still counted).
    fn is_test_file(&self, path: &str) -> bool {
        path.contains("test_") || path.contains("/tests/")
    }

    /// Language-specific AST-level skip rules (default: keep everything).
    fn should_skip_node(&self, _node: Node, _source: &[u8]) -> bool {
        false
    }

    /// Public/exported symbols, skipped unless --include-public.
    fn is_public(&self, _node: Node, _source: &[u8]) -> bool {
        false
    }
}

pub fn get_language_for_extension(ext: &str) -> Option<&'static dyn LanguageSupport> {
    static RUST: rust::RustLanguage = rust::RustLanguage;
    static PYTHON: python::PythonLanguage = python::PythonLanguage;
    static TYPESCRIPT: typescript::TypeScriptLanguage =
        typescript::TypeScriptLanguage { tsx: false };
    static TSX: typescript::TypeScriptLanguage = typescript::TypeScriptLanguage { tsx: true };
    static JAVASCRIPT: javascript::JavaScriptLanguage = javascript::JavaScriptLanguage;
    static GO: go::GoLanguage = go::GoLanguage;
    static JAVA: java::JavaLanguage = java::JavaLanguage;
    static CSHARP: csharp::CSharpLanguage = csharp::CSharpLanguage;

    match ext {
        "rs" => Some(&RUST),
        "py" => Some(&PYTHON),
        "ts" | "mts" | "cts" => Some(&TYPESCRIPT),
        "tsx" => Some(&TSX),
        "js" | "mjs" | "cjs" | "jsx" => Some(&JAVASCRIPT),
        "go" => Some(&GO),
        "java" => Some(&JAVA),
        "cs" => Some(&CSHARP),
        _ => None,
    }
}
