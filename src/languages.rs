pub mod bash;
pub mod csharp;
pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod lua;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod typescript;

use std::path::Path;

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

    /// Normalize a captured reference node's text into a symbol name; None drops it.
    fn normalize_reference(&self, _kind: &str, text: &str) -> Option<String> {
        Some(text.to_string())
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
    static RUBY: ruby::RubyLanguage = ruby::RubyLanguage;
    static PHP: php::PhpLanguage = php::PhpLanguage;
    static BASH: bash::BashLanguage = bash::BashLanguage;
    static KOTLIN: kotlin::KotlinLanguage = kotlin::KotlinLanguage;
    static LUA: lua::LuaLanguage = lua::LuaLanguage;

    match ext {
        "rs" => Some(&RUST),
        "py" => Some(&PYTHON),
        "ts" | "mts" | "cts" => Some(&TYPESCRIPT),
        "tsx" => Some(&TSX),
        "js" | "mjs" | "cjs" | "jsx" => Some(&JAVASCRIPT),
        "go" => Some(&GO),
        "java" => Some(&JAVA),
        "cs" => Some(&CSHARP),
        "rb" => Some(&RUBY),
        "php" => Some(&PHP),
        "sh" | "bash" => Some(&BASH),
        "kt" | "kts" => Some(&KOTLIN),
        "lua" => Some(&LUA),
        _ => None,
    }
}

/// Map a shebang line to a canonical extension understood by `get_language_for_extension`.
pub fn shebang_extension(first_line: &str) -> Option<&'static str> {
    let rest = first_line.strip_prefix("#!")?;
    let mut tokens = rest.split_whitespace();
    let mut interpreter = tokens.next()?.rsplit('/').next()?;
    if interpreter == "env" {
        // Skip env flags (e.g. -S) and VAR=value assignments to find the real interpreter.
        interpreter = tokens.find(|t| !t.starts_with('-') && !t.contains('='))?;
    }
    let base = interpreter.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    match base {
        "sh" | "bash" | "dash" | "ksh" | "zsh" => Some("sh"),
        "python" => Some("py"),
        "ruby" => Some("rb"),
        "node" | "nodejs" => Some("js"),
        "lua" => Some("lua"),
        "php" => Some("php"),
        _ => None,
    }
}

/// Resolve a file to a language: by extension if it has one, otherwise by shebang sniffing.
pub fn get_language_for_file(path: &Path) -> Option<&'static dyn LanguageSupport> {
    if let Some(ext) = path.extension() {
        return get_language_for_extension(&ext.to_string_lossy());
    }

    let mut buf = [0u8; 256];
    let n = std::fs::File::open(path)
        .and_then(|mut f| std::io::Read::read(&mut f, &mut buf))
        .ok()?;
    let head = &buf[..n];
    if !head.starts_with(b"#!") {
        return None;
    }
    let text = String::from_utf8_lossy(head);
    let first_line = text.lines().next()?;
    shebang_extension(first_line).and_then(get_language_for_extension)
}
