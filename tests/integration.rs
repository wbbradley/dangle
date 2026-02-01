use std::{collections::HashMap, path::Path};

use dangle::{
    analysis::find_dead_code,
    languages::get_language_for_extension,
    symbols::{Definition, extract_definitions, extract_references},
};

/// Helper to analyze source code and return dead code candidates
fn analyze_source(source: &str, ext: &str) -> Vec<Definition> {
    let lang = get_language_for_extension(ext).expect("unsupported extension");
    let path = Path::new(match ext {
        "rs" => "test.rs",
        "py" => "test.py",
        _ => "test.txt",
    });

    let definitions =
        extract_definitions(path, source, lang).expect("failed to extract definitions");
    let references = extract_references(source, lang).expect("failed to extract references");

    let mut ref_counts: HashMap<String, usize> = HashMap::new();
    for r in references {
        *ref_counts.entry(r.name).or_insert(0) += 1;
    }

    find_dead_code(definitions, &ref_counts).dead_code
}

/// Helper to get just the names of dead code
fn dead_names(source: &str, ext: &str) -> Vec<String> {
    analyze_source(source, ext)
        .into_iter()
        .map(|d| d.name)
        .collect()
}

// =============================================================================
// Rust Tests
// =============================================================================

#[test]
fn rust_unreferenced_function_is_dead() {
    let source = r#"
fn unused_function() {}
"#;
    let dead = dead_names(source, "rs");
    assert_eq!(dead, vec!["unused_function"]);
}

#[test]
fn rust_referenced_function_is_not_dead() {
    let source = r#"
fn used_function() {}

fn caller() {
    used_function();
}
"#;
    let dead = dead_names(source, "rs");
    // caller is dead (only referenced once at definition), but used_function is not
    assert!(!dead.contains(&"used_function".to_string()));
    assert!(dead.contains(&"caller".to_string()));
}

#[test]
fn rust_impl_drop_is_not_dead() {
    let source = r#"
struct MyResource {
    handle: i32,
}

impl Drop for MyResource {
    fn drop(&mut self) {
        // cleanup
    }
}
"#;
    let dead = dead_names(source, "rs");
    // drop should be ignored by should_ignore
    assert!(!dead.contains(&"drop".to_string()));
}

#[test]
fn rust_main_is_not_dead() {
    let source = r#"
fn main() {
    println!("hello");
}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"main".to_string()));
}

#[test]
fn rust_test_functions_are_not_dead() {
    let source = r#"
#[test]
fn test_something() {
    assert!(true);
}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"test_something".to_string()));
}

#[test]
fn rust_nodangle_comment_excludes_function() {
    let source = r#"
fn intentionally_unused() {} // nodangle
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"intentionally_unused".to_string()));
}

#[test]
fn rust_struct_unreferenced_is_dead() {
    let source = r#"
struct UnusedStruct {
    field: i32,
}
"#;
    let dead = dead_names(source, "rs");
    assert!(dead.contains(&"UnusedStruct".to_string()));
}

#[test]
fn rust_struct_referenced_is_not_dead() {
    let source = r#"
struct UsedStruct {
    field: i32,
}

fn use_it() -> UsedStruct {
    UsedStruct { field: 42 }
}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"UsedStruct".to_string()));
}

#[test]
fn rust_const_unreferenced_is_dead() {
    let source = r#"
const UNUSED_CONST: i32 = 42;
"#;
    let dead = dead_names(source, "rs");
    assert!(dead.contains(&"UNUSED_CONST".to_string()));
}

#[test]
fn rust_enum_unreferenced_is_dead() {
    let source = r#"
enum UnusedEnum {
    A,
    B,
}
"#;
    let dead = dead_names(source, "rs");
    assert!(dead.contains(&"UnusedEnum".to_string()));
}

#[test]
fn rust_dunder_functions_are_ignored() {
    let source = r#"
fn __private_internal() {}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"__private_internal".to_string()));
}

#[test]
fn rust_column_numbers_are_correct() {
    let source = r#"fn foo() {}"#;
    let dead = analyze_source(source, "rs");
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].name, "foo");
    assert_eq!(dead[0].line, 1);
    assert_eq!(dead[0].column, 4); // "fn " is 3 chars, so column 4 (1-indexed)
}

// =============================================================================
// Python Tests
// =============================================================================

#[test]
fn python_unreferenced_function_is_dead() {
    let source = r#"
def unused_function():
    pass
"#;
    let dead = dead_names(source, "py");
    assert!(dead.contains(&"unused_function".to_string()));
}

#[test]
fn python_referenced_function_is_not_dead() {
    let source = r#"
def used_function():
    pass

def caller():
    used_function()
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"used_function".to_string()));
}

#[test]
fn python_main_is_not_dead() {
    let source = r#"
def main():
    print("hello")
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"main".to_string()));
}

#[test]
fn python_test_functions_are_not_dead() {
    let source = r#"
def test_something():
    assert True
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"test_something".to_string()));
}

#[test]
fn python_dunder_methods_are_ignored() {
    let source = r#"
class MyClass:
    def __init__(self):
        pass

    def __str__(self):
        return "MyClass"
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"__init__".to_string()));
    assert!(!dead.contains(&"__str__".to_string()));
}

#[test]
fn python_class_unreferenced_is_dead() {
    let source = r#"
class UnusedClass:
    pass
"#;
    let dead = dead_names(source, "py");
    assert!(dead.contains(&"UnusedClass".to_string()));
}

#[test]
fn python_class_referenced_is_not_dead() {
    let source = r#"
class UsedClass:
    pass

def use_it():
    return UsedClass()
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"UsedClass".to_string()));
}

#[test]
fn python_nodangle_comment_excludes_function() {
    let source = r#"
def intentionally_unused():  # nodangle
    pass
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"intentionally_unused".to_string()));
}

#[test]
fn python_module_level_variable_unreferenced_is_dead() {
    let source = r#"
UNUSED_VAR = 42
"#;
    let dead = dead_names(source, "py");
    assert!(dead.contains(&"UNUSED_VAR".to_string()));
}

#[test]
fn python_column_numbers_are_correct() {
    let source = r#"def foo(): pass"#;
    let dead = analyze_source(source, "py");
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].name, "foo");
    assert_eq!(dead[0].line, 1);
    assert_eq!(dead[0].column, 5); // "def " is 4 chars, so column 5 (1-indexed)
}
