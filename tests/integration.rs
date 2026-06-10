use std::{collections::HashMap, path::Path};

use dangle::{
    analysis::find_dead_code,
    languages::get_language_for_extension,
    symbols::{Definition, extract_definitions, extract_references},
};

/// Helper to analyze source code and return dead code candidates
fn analyze_source_with(source: &str, ext: &str, include_public: bool) -> Vec<Definition> {
    let lang = get_language_for_extension(ext).expect("unsupported extension");
    let path_str = format!("source.{ext}");
    let path = Path::new(&path_str);

    let definitions = extract_definitions(path, source, lang, include_public)
        .expect("failed to extract definitions");
    let references = extract_references(source, lang).expect("failed to extract references");

    let mut ref_counts: HashMap<String, usize> = HashMap::new();
    for r in references {
        *ref_counts.entry(r.name).or_insert(0) += 1;
    }

    find_dead_code(definitions, &ref_counts).dead_code
}

/// Helper to analyze source code and return dead code candidates (public symbols skipped)
fn analyze_source(source: &str, ext: &str) -> Vec<Definition> {
    analyze_source_with(source, ext, false)
}

/// Helper to get just the names of dead code
fn dead_names(source: &str, ext: &str) -> Vec<String> {
    analyze_source(source, ext)
        .into_iter()
        .map(|d| d.name)
        .collect()
}

/// Helper to get dead code names with --include-public semantics
fn dead_names_public(source: &str, ext: &str) -> Vec<String> {
    analyze_source_with(source, ext, true)
        .into_iter()
        .map(|d| d.name)
        .collect()
}

// =============================================================================
// Rust Tests
// =============================================================================

#[test]
fn test_rust_unreferenced_function_is_dead() {
    let source = r#"
fn unused_function() {}
"#;
    let dead = dead_names(source, "rs");
    assert_eq!(dead, vec!["unused_function"]);
}

#[test]
fn test_rust_referenced_function_is_not_dead() {
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
fn test_rust_impl_drop_is_not_dead() {
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
    // drop is inside a trait impl, so it should be ignored
    assert!(!dead.contains(&"drop".to_string()));
}

#[test]
fn test_rust_trait_impl_functions_are_not_dead() {
    let source = r#"
trait MyTrait {
    fn do_something(&self);
    fn do_another(&self);
}

struct MyStruct;

impl MyTrait for MyStruct {
    fn do_something(&self) {
        println!("something");
    }

    fn do_another(&self) {
        println!("another");
    }
}
"#;
    let dead = dead_names(source, "rs");
    // Functions inside trait impls should be ignored
    assert!(!dead.contains(&"do_something".to_string()));
    assert!(!dead.contains(&"do_another".to_string()));
}

#[test]
fn test_rust_inherent_impl_functions_are_detected() {
    let source = r#"
struct MyStruct;

impl MyStruct {
    fn unused_method(&self) {
        println!("unused");
    }
}
"#;
    let dead = dead_names(source, "rs");
    // Functions in inherent impls (not trait impls) should still be detected
    assert!(dead.contains(&"unused_method".to_string()));
}

#[test]
fn test_rust_method_call_counts_as_reference() {
    let source = r#"
struct MyStruct;

impl MyStruct {
    fn used_method(&self) {
        println!("used");
    }
}

fn caller() {
    let s = MyStruct;
    s.used_method();
}
"#;
    let dead = dead_names(source, "rs");
    // Method calls (field_identifier) should count as references
    assert!(!dead.contains(&"used_method".to_string()));
}

#[test]
fn test_rust_private_trait_unreferenced_is_dead() {
    let source = r#"
trait UnusedTrait {
    fn do_something(&self);
}
"#;
    let dead = dead_names(source, "rs");
    // Private traits with no implementations should be flagged
    assert!(dead.contains(&"UnusedTrait".to_string()));
}

#[test]
fn test_rust_public_trait_is_not_dead() {
    let source = r#"
pub trait ExportedTrait {
    fn do_something(&self);
}
"#;
    let dead = dead_names(source, "rs");
    // Public traits should not be flagged (they may be implemented downstream)
    assert!(!dead.contains(&"ExportedTrait".to_string()));
}

#[test]
fn test_rust_private_trait_with_impl_is_not_dead() {
    let source = r#"
trait InternalTrait {
    fn do_something(&self);
}

struct MyStruct;

impl InternalTrait for MyStruct {
    fn do_something(&self) {
        println!("something");
    }
}
"#;
    let dead = dead_names(source, "rs");
    // Private traits that are implemented should not be flagged
    assert!(!dead.contains(&"InternalTrait".to_string()));
}

#[test]
fn test_rust_main_is_not_dead() {
    let source = r#"
fn main() {
    println!("hello");
}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"main".to_string()));
}

#[test]
fn test_rust_test_functions_are_not_dead() {
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
fn test_rust_test_attribute_ignores_function() {
    let source = r#"
#[test]
fn some_test_without_prefix() {
    assert!(true);
}
"#;
    let dead = dead_names(source, "rs");
    // Functions with #[test] attribute should be ignored even without test_ prefix
    assert!(!dead.contains(&"some_test_without_prefix".to_string()));
}

#[test]
fn test_rust_allow_unused_ignores_function() {
    let source = r#"
#[allow(unused)]
fn intentionally_unused_fn() {}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"intentionally_unused_fn".to_string()));
}

#[test]
fn test_rust_allow_dead_code_ignores_function() {
    let source = r#"
#[allow(dead_code)]
fn dead_code_fn() {}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"dead_code_fn".to_string()));
}

#[test]
fn test_rust_nodangle_comment_excludes_function() {
    let source = r#"
fn intentionally_unused() {} // nodangle
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"intentionally_unused".to_string()));
}

#[test]
fn test_rust_struct_unreferenced_is_dead() {
    let source = r#"
struct UnusedStruct {
    field: i32,
}
"#;
    let dead = dead_names(source, "rs");
    assert!(dead.contains(&"UnusedStruct".to_string()));
}

#[test]
fn test_rust_struct_referenced_is_not_dead() {
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
fn test_rust_const_unreferenced_is_dead() {
    let source = r#"
const UNUSED_CONST: i32 = 42;
"#;
    let dead = dead_names(source, "rs");
    assert!(dead.contains(&"UNUSED_CONST".to_string()));
}

#[test]
fn test_rust_enum_unreferenced_is_dead() {
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
fn test_rust_dunder_functions_are_ignored() {
    let source = r#"
fn __private_internal() {}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"__private_internal".to_string()));
}

#[test]
fn test_rust_column_numbers_are_correct() {
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
fn test_python_unreferenced_function_is_dead() {
    let source = r#"
def unused_function():
    pass
"#;
    let dead = dead_names(source, "py");
    assert!(dead.contains(&"unused_function".to_string()));
}

#[test]
fn test_python_referenced_function_is_not_dead() {
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
fn test_python_main_is_not_dead() {
    let source = r#"
def main():
    print("hello")
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"main".to_string()));
}

#[test]
fn test_python_test_functions_are_not_dead() {
    let source = r#"
def test_something():
    assert True
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"test_something".to_string()));
}

#[test]
fn test_python_test_classes_are_not_dead() {
    let source = r#"
class TestMyFeature:
    def test_something(self):
        assert True
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"TestMyFeature".to_string()));
}

#[test]
fn test_python_dunder_methods_are_ignored() {
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
fn test_python_class_unreferenced_is_dead() {
    let source = r#"
class UnusedClass:
    pass
"#;
    let dead = dead_names(source, "py");
    assert!(dead.contains(&"UnusedClass".to_string()));
}

#[test]
fn test_python_class_referenced_is_not_dead() {
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
fn test_python_nodangle_comment_excludes_function() {
    let source = r#"
def intentionally_unused():  # nodangle
    pass
"#;
    let dead = dead_names(source, "py");
    assert!(!dead.contains(&"intentionally_unused".to_string()));
}

#[test]
fn test_python_module_level_variable_unreferenced_is_dead() {
    let source = r#"
UNUSED_VAR = 42
"#;
    let dead = dead_names(source, "py");
    assert!(dead.contains(&"UNUSED_VAR".to_string()));
}

#[test]
fn test_python_column_numbers_are_correct() {
    let source = r#"def foo(): pass"#;
    let dead = analyze_source(source, "py");
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].name, "foo");
    assert_eq!(dead[0].line, 1);
    assert_eq!(dead[0].column, 5); // "def " is 4 chars, so column 5 (1-indexed)
}

// =============================================================================
// Rust Attribute String Reference Tests
// =============================================================================

#[test]
fn test_rust_attribute_string_counts_as_reference() {
    let source = r#"
fn default_value() -> i32 { 42 }

#[derive(Default)]
struct Config {
    #[serde(default = "default_value")]
    value: i32,
}
"#;
    let dead = dead_names(source, "rs");
    assert!(!dead.contains(&"default_value".to_string()));
}

#[test]
fn test_rust_attribute_string_invalid_identifier_not_counted() {
    let source = r#"
fn unused_fn() {}

#[some_attr(key = "not-a-valid-identifier")]
struct Config {
    value: i32,
}
"#;
    let dead = dead_names(source, "rs");
    // "not-a-valid-identifier" contains hyphens, so it should not count as a reference
    // unused_fn should still be dead
    assert!(dead.contains(&"unused_fn".to_string()));
}

#[test]
fn test_rust_attribute_string_path_extracts_leaf() {
    let source = r#"
fn default_monitoring() -> bool { true }

#[derive(Default)]
struct Config {
    #[serde(default = "defaults::default_monitoring")]
    monitoring: bool,
}
"#;
    let dead = dead_names(source, "rs");
    // Should extract "default_monitoring" from "defaults::default_monitoring"
    assert!(!dead.contains(&"default_monitoring".to_string()));
}

#[test]
fn test_rust_public_trait_reported_with_include_public() {
    let source = r#"
pub trait ExportedTrait {
    fn do_something(&self);
}
"#;
    let dead = dead_names_public(source, "rs");
    assert!(dead.contains(&"ExportedTrait".to_string()));
}

// =============================================================================
// TypeScript Tests
// =============================================================================

#[test]
fn test_typescript_unreferenced_function_is_dead() {
    let source = r#"
function unusedFunction() {}
"#;
    let dead = dead_names(source, "ts");
    assert!(dead.contains(&"unusedFunction".to_string()));
}

#[test]
fn test_typescript_referenced_function_is_not_dead() {
    let source = r#"
function usedFunction() {}

function caller() {
    usedFunction();
}
"#;
    let dead = dead_names(source, "ts");
    assert!(!dead.contains(&"usedFunction".to_string()));
    assert!(dead.contains(&"caller".to_string()));
}

#[test]
fn test_typescript_unreferenced_class_is_dead() {
    let source = r#"
class UnusedClass {}
"#;
    let dead = dead_names(source, "ts");
    assert!(dead.contains(&"UnusedClass".to_string()));
}

#[test]
fn test_typescript_referenced_class_is_not_dead() {
    let source = r#"
class UsedClass {}

function makeIt() {
    return new UsedClass();
}
"#;
    let dead = dead_names(source, "ts");
    assert!(!dead.contains(&"UsedClass".to_string()));
}

#[test]
fn test_typescript_unreferenced_interface_is_dead() {
    let source = r#"
interface UnusedInterface {
    field: number;
}
"#;
    let dead = dead_names(source, "ts");
    assert!(dead.contains(&"UnusedInterface".to_string()));
}

#[test]
fn test_typescript_interface_used_in_type_annotation_is_not_dead() {
    let source = r#"
interface Config {
    field: number;
}

function useIt(config: Config) {
    return config.field;
}
"#;
    let dead = dead_names(source, "ts");
    assert!(!dead.contains(&"Config".to_string()));
}

#[test]
fn test_typescript_unreferenced_type_alias_is_dead() {
    let source = r#"
type UnusedAlias = string | number;
"#;
    let dead = dead_names(source, "ts");
    assert!(dead.contains(&"UnusedAlias".to_string()));
}

#[test]
fn test_typescript_unreferenced_enum_is_dead() {
    let source = r#"
enum UnusedEnum {
    A,
    B,
}
"#;
    let dead = dead_names(source, "ts");
    assert!(dead.contains(&"UnusedEnum".to_string()));
}

#[test]
fn test_typescript_unreferenced_top_level_const_is_dead() {
    let source = r#"
const unusedConst = 42;
"#;
    let dead = dead_names(source, "ts");
    assert!(dead.contains(&"unusedConst".to_string()));
}

#[test]
fn test_typescript_method_call_counts_as_reference() {
    let source = r#"
function helper() {}

const obj = {
    helper,
};

function caller(o: typeof obj) {
    o.helper();
}
"#;
    let dead = dead_names(source, "ts");
    // property_identifier and shorthand_property_identifier refs keep helper live
    assert!(!dead.contains(&"helper".to_string()));
}

#[test]
fn test_typescript_property_access_keeps_method_named_function_live() {
    let source = r#"
function render() {}

const api = { draw: 1 };

function caller() {
    api.render();
}
"#;
    let dead = dead_names(source, "ts");
    // obj.method() usage (property_identifier) counts as a reference
    assert!(!dead.contains(&"render".to_string()));
}

#[test]
fn test_typescript_exported_function_skipped_by_default() {
    let source = r#"
export function publicApi() {}
"#;
    let dead = dead_names(source, "ts");
    assert!(!dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_typescript_exported_function_reported_with_include_public() {
    let source = r#"
export function publicApi() {}
"#;
    let dead = dead_names_public(source, "ts");
    assert!(dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_typescript_exported_const_skipped_by_default() {
    let source = r#"
export const publicConst = 42;
"#;
    let dead = dead_names(source, "ts");
    assert!(!dead.contains(&"publicConst".to_string()));
}

#[test]
fn test_typescript_exported_const_reported_with_include_public() {
    let source = r#"
export const publicConst = 42;
"#;
    let dead = dead_names_public(source, "ts");
    assert!(dead.contains(&"publicConst".to_string()));
}

#[test]
fn test_typescript_underscore_prefixed_names_are_ignored() {
    let source = r#"
function _internalHelper() {}
"#;
    let dead = dead_names(source, "ts");
    assert!(!dead.contains(&"_internalHelper".to_string()));
}

// =============================================================================
// TSX Tests
// =============================================================================

#[test]
fn test_tsx_component_referenced_as_jsx_element_is_not_dead() {
    let source = r#"
function MyComponent() {
    return <div />;
}

function App() {
    return <MyComponent />;
}
"#;
    let dead = dead_names(source, "tsx");
    assert!(!dead.contains(&"MyComponent".to_string()));
}

#[test]
fn test_tsx_unused_component_is_dead() {
    let source = r#"
function UnusedComponent() {
    return <div />;
}
"#;
    let dead = dead_names(source, "tsx");
    assert!(dead.contains(&"UnusedComponent".to_string()));
}

// =============================================================================
// JavaScript Tests
// =============================================================================

#[test]
fn test_javascript_unreferenced_function_is_dead() {
    let source = r#"
function unusedFunction() {}
"#;
    let dead = dead_names(source, "js");
    assert!(dead.contains(&"unusedFunction".to_string()));
}

#[test]
fn test_javascript_unreferenced_class_is_dead() {
    let source = r#"
class UnusedClass {}
"#;
    let dead = dead_names(source, "js");
    assert!(dead.contains(&"UnusedClass".to_string()));
}

#[test]
fn test_javascript_referenced_function_is_not_dead() {
    let source = r#"
function usedFunction() {}

function caller() {
    usedFunction();
}
"#;
    let dead = dead_names(source, "js");
    assert!(!dead.contains(&"usedFunction".to_string()));
}

#[test]
fn test_javascript_exported_symbol_skipped_by_default() {
    let source = r#"
export function publicApi() {}
export const publicConst = 1;
"#;
    let dead = dead_names(source, "js");
    assert!(!dead.contains(&"publicApi".to_string()));
    assert!(!dead.contains(&"publicConst".to_string()));
}

#[test]
fn test_javascript_exported_symbol_reported_with_include_public() {
    let source = r#"
export function publicApi() {}
"#;
    let dead = dead_names_public(source, "js");
    assert!(dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_jsx_usage_counts_as_reference() {
    let source = r#"
function MyComponent() {
    return <div />;
}

function App() {
    return <MyComponent />;
}
"#;
    let dead = dead_names(source, "jsx");
    assert!(!dead.contains(&"MyComponent".to_string()));
}

// =============================================================================
// Go Tests
// =============================================================================

#[test]
fn test_go_unreferenced_unexported_function_is_dead() {
    let source = r#"
package main

func unusedHelper() {}
"#;
    let dead = dead_names(source, "go");
    assert!(dead.contains(&"unusedHelper".to_string()));
}

#[test]
fn test_go_referenced_function_is_not_dead() {
    let source = r#"
package main

func usedHelper() {}

func caller() {
	usedHelper()
}
"#;
    let dead = dead_names(source, "go");
    assert!(!dead.contains(&"usedHelper".to_string()));
    assert!(dead.contains(&"caller".to_string()));
}

#[test]
fn test_go_exported_function_skipped_by_default() {
    let source = r#"
package main

func PublicApi() {}
"#;
    let dead = dead_names(source, "go");
    assert!(!dead.contains(&"PublicApi".to_string()));
}

#[test]
fn test_go_exported_function_reported_with_include_public() {
    let source = r#"
package main

func PublicApi() {}
"#;
    let dead = dead_names_public(source, "go");
    assert!(dead.contains(&"PublicApi".to_string()));
}

#[test]
fn test_go_main_and_init_are_not_dead() {
    let source = r#"
package main

func main() {}

func init() {}
"#;
    let dead = dead_names(source, "go");
    assert!(!dead.contains(&"main".to_string()));
    assert!(!dead.contains(&"init".to_string()));
}

#[test]
fn test_go_unused_method_is_dead() {
    let source = r#"
package main

type widget struct{}

func (w widget) unusedMethod() {}

func useWidget() widget {
	return widget{}
}
"#;
    let dead = dead_names(source, "go");
    assert!(dead.contains(&"unusedMethod".to_string()));
}

#[test]
fn test_go_top_level_type_const_var_detection() {
    let source = r#"
package main

type unusedType struct{}

const unusedConst = 42

var unusedVar = "hello"

var (
	unusedGrouped = 1
)
"#;
    let dead = dead_names(source, "go");
    assert!(dead.contains(&"unusedType".to_string()));
    assert!(dead.contains(&"unusedConst".to_string()));
    assert!(dead.contains(&"unusedVar".to_string()));
    assert!(dead.contains(&"unusedGrouped".to_string()));
}

#[test]
fn test_go_type_referenced_is_not_dead() {
    let source = r#"
package main

type config struct{}

func loadConfig() config {
	return config{}
}
"#;
    let dead = dead_names(source, "go");
    assert!(!dead.contains(&"config".to_string()));
}

#[test]
fn test_go_test_functions_are_ignored() {
    let source = r#"
package main

func TestFoo(t *testing.T) {}

func BenchmarkBar(b *testing.B) {}
"#;
    let dead = dead_names_public(source, "go");
    assert!(!dead.contains(&"TestFoo".to_string()));
    assert!(!dead.contains(&"BenchmarkBar".to_string()));
}

#[test]
fn test_go_test_file_detection() {
    let lang = get_language_for_extension("go").unwrap();
    assert!(lang.is_test_file("pkg/foo_test.go"));
    assert!(!lang.is_test_file("pkg/foo.go"));
}

// =============================================================================
// Java Tests
// =============================================================================

#[test]
fn test_java_unused_private_method_is_dead() {
    let source = r#"
class App {
    private void unusedHelper() {}
}
"#;
    let dead = dead_names(source, "java");
    assert!(dead.contains(&"unusedHelper".to_string()));
}

#[test]
fn test_java_referenced_method_is_not_dead() {
    let source = r#"
class App {
    private void usedHelper() {}

    private void caller() {
        usedHelper();
    }
}
"#;
    let dead = dead_names(source, "java");
    assert!(!dead.contains(&"usedHelper".to_string()));
}

#[test]
fn test_java_public_method_skipped_by_default() {
    let source = r#"
class App {
    public void publicApi() {}
}
"#;
    let dead = dead_names(source, "java");
    assert!(!dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_java_public_method_reported_with_include_public() {
    let source = r#"
class App {
    public void publicApi() {}
}
"#;
    let dead = dead_names_public(source, "java");
    assert!(dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_java_test_annotated_method_is_skipped() {
    let source = r#"
class AppTest {
    @Test
    void somethingWorks() {}

    @org.junit.jupiter.api.Test
    void qualifiedAnnotation() {}
}
"#;
    let dead = dead_names(source, "java");
    assert!(!dead.contains(&"somethingWorks".to_string()));
    assert!(!dead.contains(&"qualifiedAnnotation".to_string()));
}

#[test]
fn test_java_override_method_is_skipped() {
    let source = r#"
class App {
    @Override
    protected void onStart() {}
}
"#;
    let dead = dead_names(source, "java");
    assert!(!dead.contains(&"onStart".to_string()));
}

#[test]
fn test_java_tostring_is_ignored() {
    let source = r#"
class App {
    private String toString(int x) { return ""; }
}
"#;
    let dead = dead_names(source, "java");
    assert!(!dead.contains(&"toString".to_string()));
}

#[test]
fn test_java_unused_class_is_dead() {
    let source = r#"
class UnusedClass {}
"#;
    let dead = dead_names(source, "java");
    assert!(dead.contains(&"UnusedClass".to_string()));
}

#[test]
fn test_java_class_referenced_as_type_is_not_dead() {
    let source = r#"
class Foo {}

class App {
    private void useIt() {
        Foo x = new Foo();
    }
}
"#;
    let dead = dead_names(source, "java");
    assert!(!dead.contains(&"Foo".to_string()));
}

#[test]
fn test_java_unused_field_is_dead() {
    let source = r#"
class App {
    private int unusedField;
}
"#;
    let dead = dead_names(source, "java");
    assert!(dead.contains(&"unusedField".to_string()));
}

#[test]
fn test_java_test_file_detection() {
    let lang = get_language_for_extension("java").unwrap();
    assert!(lang.is_test_file("src/test/java/com/example/AppTest.java"));
    assert!(lang.is_test_file("src/main/java/AppTest.java"));
    assert!(lang.is_test_file("TestHelpers.java"));
    assert!(!lang.is_test_file("src/main/java/App.java"));
}

// =============================================================================
// C# Tests
// =============================================================================

#[test]
fn test_csharp_unused_private_method_is_dead() {
    let source = r#"
class App {
    private void UnusedHelper() {}
}
"#;
    let dead = dead_names(source, "cs");
    assert!(dead.contains(&"UnusedHelper".to_string()));
}

#[test]
fn test_csharp_referenced_method_is_not_dead() {
    let source = r#"
class App {
    private void UsedHelper() {}

    private void Caller() {
        UsedHelper();
    }
}
"#;
    let dead = dead_names(source, "cs");
    assert!(!dead.contains(&"UsedHelper".to_string()));
}

#[test]
fn test_csharp_public_member_skipped_by_default() {
    let source = r#"
public class App {
    public void PublicApi() {}
    public int PublicProp { get; set; }
}
"#;
    let dead = dead_names(source, "cs");
    assert!(!dead.contains(&"App".to_string()));
    assert!(!dead.contains(&"PublicApi".to_string()));
    assert!(!dead.contains(&"PublicProp".to_string()));
}

#[test]
fn test_csharp_public_member_reported_with_include_public() {
    let source = r#"
public class App {
    public void PublicApi() {}
}
"#;
    let dead = dead_names_public(source, "cs");
    assert!(dead.contains(&"App".to_string()));
    assert!(dead.contains(&"PublicApi".to_string()));
}

#[test]
fn test_csharp_fact_attributed_method_is_skipped() {
    let source = r#"
class AppTests {
    [Fact]
    void SomethingWorks() {}

    [Xunit.Theory]
    void TheoryCase() {}
}
"#;
    let dead = dead_names(source, "cs");
    assert!(!dead.contains(&"SomethingWorks".to_string()));
    assert!(!dead.contains(&"TheoryCase".to_string()));
}

#[test]
fn test_csharp_main_is_ignored() {
    let source = r#"
class Program {
    static void Main(string[] args) {}
}
"#;
    let dead = dead_names(source, "cs");
    assert!(!dead.contains(&"Main".to_string()));
}

#[test]
fn test_csharp_unreferenced_partial_class_is_dead() {
    let source = r#"
partial class Config {
    private int x;
}

partial class Config {
    private int y;
}
"#;
    let dead = dead_names(source, "cs");
    // Two definitions of the same name must not keep each other alive
    assert!(dead.contains(&"Config".to_string()));
}

#[test]
fn test_csharp_referenced_partial_class_is_not_dead() {
    let source = r#"
partial class Config {}

partial class Config {}

class App {
    private Config MakeConfig() {
        return new Config();
    }
}
"#;
    let dead = dead_names(source, "cs");
    assert!(!dead.contains(&"Config".to_string()));
}

#[test]
fn test_csharp_test_file_detection() {
    let lang = get_language_for_extension("cs").unwrap();
    assert!(lang.is_test_file("src/AppTest.cs"));
    assert!(lang.is_test_file("src/AppTests.cs"));
    assert!(lang.is_test_file("proj/tests/Helpers.cs"));
    assert!(!lang.is_test_file("src/App.cs"));
}

// =============================================================================
// Same-name Definition Masking Regression Tests
// =============================================================================

#[test]
fn test_rust_same_name_fns_in_different_modules_are_dead() {
    let source = r#"
mod alpha {
    fn helper() {}
}

mod beta {
    fn helper() {}
}
"#;
    let dead = dead_names(source, "rs");
    // Two unused same-name definitions must not count each other as references
    assert_eq!(
        dead.iter().filter(|n| n.as_str() == "helper").count(),
        2,
        "both unused same-name fns should be reported"
    );
}

// =============================================================================
// Test-file Detection Tests
// =============================================================================

#[test]
fn test_typescript_test_file_detection() {
    let lang = get_language_for_extension("ts").unwrap();
    assert!(lang.is_test_file("src/foo.test.ts"));
    assert!(lang.is_test_file("src/__tests__/foo.ts"));
    assert!(lang.is_test_file("src/types/foo.d.ts"));
    assert!(!lang.is_test_file("src/foo.ts"));

    let tsx = get_language_for_extension("tsx").unwrap();
    assert!(tsx.is_test_file("src/foo.spec.tsx"));
    assert!(!tsx.is_test_file("src/foo.tsx"));
}

#[test]
fn test_javascript_test_file_detection() {
    let lang = get_language_for_extension("js").unwrap();
    assert!(lang.is_test_file("src/foo.test.js"));
    assert!(lang.is_test_file("src/foo.spec.js"));
    assert!(lang.is_test_file("src/__tests__/foo.js"));
    assert!(!lang.is_test_file("src/foo.js"));
}

#[test]
fn test_rust_and_python_test_file_detection_defaults() {
    let rust = get_language_for_extension("rs").unwrap();
    assert!(rust.is_test_file("crate/tests/integration.rs"));
    assert!(rust.is_test_file("src/test_helpers.rs"));
    assert!(!rust.is_test_file("src/main.rs"));

    let python = get_language_for_extension("py").unwrap();
    assert!(python.is_test_file("test_foo.py"));
    assert!(!python.is_test_file("foo.py"));
}

// =============================================================================
// Ruby Tests
// =============================================================================

#[test]
fn test_ruby_unreferenced_method_is_dead() {
    let source = r#"
def unused_method
end
"#;
    let dead = dead_names(source, "rb");
    assert!(dead.contains(&"unused_method".to_string()));
}

#[test]
fn test_ruby_referenced_method_is_not_dead() {
    let source = r#"
def used_method
end

def caller
  used_method
end
"#;
    let dead = dead_names(source, "rb");
    assert!(!dead.contains(&"used_method".to_string()));
    assert!(dead.contains(&"caller".to_string()));
}

#[test]
fn test_ruby_send_symbol_counts_as_reference() {
    let source = r#"
def dispatched
end

def caller
  obj.send :dispatched
end
"#;
    let dead = dead_names(source, "rb");
    assert!(!dead.contains(&"dispatched".to_string()));
}

#[test]
fn test_ruby_define_method_string_counts_as_reference() {
    let source = r#"
def fallback
end

define_method("fallback") do
end
"#;
    let dead = dead_names(source, "rb");
    assert!(!dead.contains(&"fallback".to_string()));
}

#[test]
fn test_ruby_attr_accessor_symbol_counts_as_reference() {
    let source = r#"
class Widget
  attr_accessor :size

  def size
    @size
  end
end
"#;
    let dead = dead_names(source, "rb");
    assert!(!dead.contains(&"size".to_string()));
}

#[test]
fn test_ruby_hash_key_symbol_counts_as_reference() {
    let source = r#"
def on_save
end

CALLBACKS = { on_save: true }
"#;
    let dead = dead_names(source, "rb");
    assert!(!dead.contains(&"on_save".to_string()));
}

#[test]
fn test_ruby_class_module_and_scoped_class_definitions() {
    let source = r#"
class UnusedClass
end

module UnusedModule
end

class Outer::UnusedScoped
end
"#;
    let dead = dead_names(source, "rb");
    assert!(dead.contains(&"UnusedClass".to_string()));
    assert!(dead.contains(&"UnusedModule".to_string()));
    assert!(dead.contains(&"UnusedScoped".to_string()));
}

#[test]
fn test_ruby_constant_assignment_is_a_definition() {
    let source = r#"
UNUSED_CONST = 42
"#;
    let dead = dead_names(source, "rb");
    assert!(dead.contains(&"UNUSED_CONST".to_string()));
}

#[test]
fn test_ruby_initialize_and_method_missing_are_ignored() {
    let source = r#"
class Widget
  def initialize
  end

  def method_missing(name, *args)
  end

  def respond_to_missing?(name, include_private = false)
    true
  end
end
"#;
    let dead = dead_names(source, "rb");
    assert!(!dead.contains(&"initialize".to_string()));
    assert!(!dead.contains(&"method_missing".to_string()));
    assert!(!dead.contains(&"respond_to_missing?".to_string()));
}

#[test]
fn test_ruby_send_helper_capture_is_not_a_reference() {
    // The @_m helper capture for `send` must not record `send` itself as a
    // reference: a method literally named `send_report` stays dead, and the
    // query doesn't crash on dynamic-dispatch calls.
    let source = r#"
def unused_method
end

def caller
  obj.send "something_else"
end
"#;
    let dead = dead_names(source, "rb");
    assert!(dead.contains(&"unused_method".to_string()));
}

#[test]
fn test_ruby_test_file_detection() {
    let lang = get_language_for_extension("rb").unwrap();
    assert!(lang.is_test_file("foo_spec.rb"));
    assert!(lang.is_test_file("spec/foo.rb"));
    assert!(lang.is_test_file("test/foo.rb"));
    assert!(lang.is_test_file("app/models/widget_test.rb"));
    assert!(!lang.is_test_file("lib/foo.rb"));
}

// =============================================================================
// PHP Tests
// =============================================================================

#[test]
fn test_php_unreferenced_function_is_dead() {
    let source = r#"<?php
function unused_function() {}
"#;
    let dead = dead_names(source, "php");
    assert!(dead.contains(&"unused_function".to_string()));
}

#[test]
fn test_php_referenced_function_is_not_dead() {
    let source = r#"<?php
function used_function() {}

function caller() {
    used_function();
}
"#;
    let dead = dead_names(source, "php");
    assert!(!dead.contains(&"used_function".to_string()));
    assert!(dead.contains(&"caller".to_string()));
}

#[test]
fn test_php_type_declarations_are_definitions() {
    let source = r#"<?php
class UnusedClass {}
interface UnusedInterface {}
trait UnusedTrait {}
enum UnusedEnum {
    case UnusedCase;
}
"#;
    let dead = dead_names(source, "php");
    assert!(dead.contains(&"UnusedClass".to_string()));
    assert!(dead.contains(&"UnusedInterface".to_string()));
    assert!(dead.contains(&"UnusedTrait".to_string()));
    assert!(dead.contains(&"UnusedEnum".to_string()));
    assert!(dead.contains(&"UnusedCase".to_string()));
}

#[test]
fn test_php_public_method_skipped_by_default() {
    let source = r#"<?php
class App {
    public function publicApi() {}
}
"#;
    let dead = dead_names(source, "php");
    assert!(!dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_php_public_method_reported_with_include_public() {
    let source = r#"<?php
class App {
    public function publicApi() {}
}
"#;
    let dead = dead_names_public(source, "php");
    assert!(dead.contains(&"publicApi".to_string()));
}

#[test]
fn test_php_private_and_no_modifier_methods_are_reported() {
    let source = r#"<?php
class App {
    private function privateHelper() {}
    function bareHelper() {}
}
"#;
    let dead = dead_names(source, "php");
    assert!(dead.contains(&"privateHelper".to_string()));
    assert!(dead.contains(&"bareHelper".to_string()));
}

#[test]
fn test_php_magic_methods_are_ignored() {
    let source = r#"<?php
class App {
    public function __construct() {}
    public function __get($name) {}
}
"#;
    let dead = dead_names_public(source, "php");
    assert!(!dead.contains(&"__construct".to_string()));
    assert!(!dead.contains(&"__get".to_string()));
}

#[test]
fn test_php_unused_const_is_dead() {
    let source = r#"<?php
const UNUSED_CONST = 42;
"#;
    let dead = dead_names(source, "php");
    assert!(dead.contains(&"UNUSED_CONST".to_string()));
}

#[test]
fn test_php_type_hint_counts_as_reference() {
    let source = r#"<?php
class Config {}

function load(Config $config) {}
"#;
    let dead = dead_names(source, "php");
    assert!(!dead.contains(&"Config".to_string()));
}

#[test]
fn test_php_test_file_detection() {
    let lang = get_language_for_extension("php").unwrap();
    assert!(lang.is_test_file("src/FooTest.php"));
    assert!(lang.is_test_file("tests/Foo.php"));
    assert!(lang.is_test_file("app/tests/Foo.php"));
    assert!(!lang.is_test_file("src/Foo.php"));
}

// =============================================================================
// Bash Tests
// =============================================================================

#[test]
fn test_bash_unreferenced_function_is_dead() {
    let source = r#"
unused_function() {
  echo "never called"
}
"#;
    let dead = dead_names(source, "sh");
    assert!(dead.contains(&"unused_function".to_string()));
}

#[test]
fn test_bash_referenced_function_is_not_dead() {
    let source = r#"
used_function() {
  echo "called"
}

used_function
"#;
    let dead = dead_names(source, "sh");
    assert!(!dead.contains(&"used_function".to_string()));
}

#[test]
fn test_bash_function_passed_as_argument_is_not_dead() {
    let source = r#"
cleanup() {
  rm -f /tmp/lockfile
}

trap cleanup EXIT
"#;
    let dead = dead_names(source, "bash");
    assert!(!dead.contains(&"cleanup".to_string()));
}

#[test]
fn test_bash_main_is_ignored() {
    let source = r#"
main() {
  echo "entry point"
}
"#;
    let dead = dead_names(source, "sh");
    assert!(!dead.contains(&"main".to_string()));
}

// =============================================================================
// Kotlin Tests
// =============================================================================

#[test]
fn test_kotlin_unreferenced_private_function_is_dead() {
    let source = r#"
private fun unusedHelper() {}
"#;
    let dead = dead_names(source, "kt");
    assert!(dead.contains(&"unusedHelper".to_string()));
}

#[test]
fn test_kotlin_referenced_private_function_is_not_dead() {
    let source = r#"
private fun usedHelper() {}

private fun caller() {
    usedHelper()
}
"#;
    let dead = dead_names(source, "kt");
    assert!(!dead.contains(&"usedHelper".to_string()));
    assert!(dead.contains(&"caller".to_string()));
}

#[test]
fn test_kotlin_public_symbols_skipped_by_default() {
    let source = r#"
fun publicFunction() {}

class PublicClass
"#;
    let dead = dead_names(source, "kt");
    assert!(dead.is_empty());
}

#[test]
fn test_kotlin_public_symbols_reported_with_include_public() {
    let source = r#"
fun publicFunction() {}

class PublicClass
"#;
    let dead = dead_names_public(source, "kt");
    assert!(dead.contains(&"publicFunction".to_string()));
    assert!(dead.contains(&"PublicClass".to_string()));
}

#[test]
fn test_kotlin_internal_function_is_reportable() {
    // `internal` is repo-visible only, so like `private` it is not treated as public.
    let source = r#"
internal fun internalHelper() {}
"#;
    assert!(dead_names(source, "kt").contains(&"internalHelper".to_string()));
}

#[test]
fn test_kotlin_private_class_object_and_property_are_dead() {
    let source = r#"
private class UnusedClass

private object UnusedObject

private val unusedProperty = 42
"#;
    let dead = dead_names(source, "kt");
    assert!(dead.contains(&"UnusedClass".to_string()));
    assert!(dead.contains(&"UnusedObject".to_string()));
    assert!(dead.contains(&"unusedProperty".to_string()));
}

#[test]
fn test_kotlin_function_local_val_is_reported() {
    let source = r#"
private fun worker() {
    val unused = 1
}

private fun caller() {
    worker()
}
"#;
    let dead = dead_names(source, "kt");
    assert!(dead.contains(&"unused".to_string()));
}

#[test]
fn test_kotlin_main_is_ignored() {
    let source = r#"
fun main() {}
"#;
    let dead = dead_names_public(source, "kt");
    assert!(!dead.contains(&"main".to_string()));
}

#[test]
fn test_kotlin_test_annotated_function_is_skipped() {
    let source = r#"
import org.junit.jupiter.api.Test

class FooTests {
    @Test
    fun somethingWorks() {}

    @ParameterizedTest
    fun parameterizedCheck() {}
}
"#;
    let dead = dead_names_public(source, "kt");
    assert!(!dead.contains(&"somethingWorks".to_string()));
    assert!(!dead.contains(&"parameterizedCheck".to_string()));
}

#[test]
fn test_kotlin_override_function_is_skipped() {
    let source = r#"
private class Widget {
    override fun toString(): String = "widget"
}
"#;
    let dead = dead_names_public(source, "kt");
    assert!(!dead.contains(&"toString".to_string()));
}

#[test]
fn test_kotlin_private_type_alias_is_dead() {
    let source = r#"
private typealias Handler = (Int) -> Unit
"#;
    let dead = dead_names(source, "kt");
    assert!(dead.contains(&"Handler".to_string()));
}

#[test]
fn test_kotlin_test_file_detection() {
    let lang = get_language_for_extension("kt").unwrap();
    assert!(lang.is_test_file("src/test/kotlin/FooTest.kt"));
    assert!(lang.is_test_file("FooTest.kt"));
    assert!(lang.is_test_file("app/src/test/kotlin/Bar.kt"));
    assert!(!lang.is_test_file("src/main/kotlin/Foo.kt"));
}

// =============================================================================
// Lua Tests
// =============================================================================

#[test]
fn test_lua_unreferenced_local_function_is_dead() {
    let source = r#"
local function unused_helper() end
"#;
    let dead = dead_names(source, "lua");
    assert!(dead.contains(&"unused_helper".to_string()));
}

#[test]
fn test_lua_referenced_local_function_is_not_dead() {
    let source = r#"
local function used_helper() end

local function caller()
    used_helper()
end

caller()
"#;
    let dead = dead_names(source, "lua");
    assert!(!dead.contains(&"used_helper".to_string()));
    assert!(!dead.contains(&"caller".to_string()));
}

#[test]
fn test_lua_unreferenced_local_variable_is_dead() {
    let source = r#"
local unused_var = 1
local used_var = 2
print(used_var)
"#;
    let dead = dead_names(source, "lua");
    assert!(dead.contains(&"unused_var".to_string()));
    assert!(!dead.contains(&"used_var".to_string()));
}

#[test]
fn test_lua_module_function_skipped_by_default() {
    let source = r#"
local M = {}

function M.foo() end

return M
"#;
    let dead = dead_names(source, "lua");
    assert!(!dead.contains(&"foo".to_string()));
    let dead_public = dead_names_public(source, "lua");
    assert!(dead_public.contains(&"foo".to_string()));
}

#[test]
fn test_lua_module_function_called_in_source_is_not_dead() {
    let source = r#"
local M = {}

function M.foo() end

M.foo()

return M
"#;
    let dead = dead_names_public(source, "lua");
    assert!(!dead.contains(&"foo".to_string()));
}

#[test]
fn test_lua_method_style_function_is_public() {
    let source = r#"
local M = {}

function M:bar() end

return M
"#;
    assert!(!dead_names(source, "lua").contains(&"bar".to_string()));
    assert!(dead_names_public(source, "lua").contains(&"bar".to_string()));
}

#[test]
fn test_lua_assigned_module_function_is_captured() {
    let source = r#"
local M = {}

M.baz = function() end

return M
"#;
    let dead = dead_names_public(source, "lua");
    assert!(dead.contains(&"baz".to_string()));
}

#[test]
fn test_lua_unreferenced_global_function_is_dead() {
    let source = r#"
function helper() end
"#;
    let dead = dead_names(source, "lua");
    assert!(dead.contains(&"helper".to_string()));
}

#[test]
fn test_lua_underscore_names_are_ignored() {
    let source = r#"
local _unused = 1

local function _private_helper() end
"#;
    let dead = dead_names(source, "lua");
    assert!(dead.is_empty());
}

#[test]
fn test_lua_dot_index_reference_counts() {
    let source = r#"
local M = {}

function M.helper() end

local function caller()
    M.helper()
end

caller()
"#;
    let dead = dead_names_public(source, "lua");
    assert!(!dead.contains(&"helper".to_string()));
}

#[test]
fn test_lua_test_file_detection() {
    let lang = get_language_for_extension("lua").unwrap();
    assert!(lang.is_test_file("foo_spec.lua"));
    assert!(lang.is_test_file("spec/foo.lua"));
    assert!(lang.is_test_file("tests/foo.lua"));
    assert!(lang.is_test_file("lua/spec/bar.lua"));
    assert!(!lang.is_test_file("lua/mymodule.lua"));
}
