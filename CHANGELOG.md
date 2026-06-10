# Changelog

## [0.6.0] - 2026-06-10

### Breaking Changes

- **Library API:** `languages::LanguageSupport::normalize_reference` now returns `Vec<String>`
  instead of `Option<String>` (one capture can expand to several symbol names), and its first
  argument is the query capture name (e.g. `"macro_string"`) instead of the tree-sitter node
  kind. Implementers should return `vec![name]` / `Vec::new()` in place of `Some(name)` / `None`
  and match on the capture names used in their `references_query`. No CLI impact.

### Added

- Rust: identifiers used as inline format args in macro strings now count as references
  (e.g. `format!("query={QUERY}")`, `panic!("failed with {CODE}")`), including raw strings,
  format specs (`{value:?}`, `{value:>8}`), and width/precision args (`{x:>WIDTH$.PRECISION$}`).
  Escaped braces (`{{name}}`) and positional args (`{}`, `{0}`) are correctly ignored.

### Fixed

- Rust consts/statics referenced only via inline format args in `format!`-family macros are no
  longer falsely reported as dead code.

## [0.5.0] - 2026-06-10

### Breaking Changes

- **Library API:** `symbols::extract_definitions` now takes an `include_public: bool` argument;
  pass `false` for the previous behavior.
- **Library API:** `languages::RustLanguage` and `languages::PythonLanguage` moved to
  `languages::rust::RustLanguage` and `languages::python::PythonLanguage`.
- **Behavior:** a name defined in multiple places (e.g. C# partial classes, same-name functions
  in different modules) no longer keeps itself alive; all copies are reported unless referenced
  elsewhere. Previously suppressed dead code may now appear.
- **Behavior:** dangle now scans 11 additional languages by default plus extensionless shebang
  scripts, so runs on polyglot repos will produce findings for files that were previously
  ignored. CI setups that gate on dangle output may newly fail.

### Added

- Support for 11 new languages: TypeScript/TSX, JavaScript/JSX (incl. `.mts`/`.cts`/`.mjs`/`.cjs`),
  Go, Java, C#, Ruby, PHP, Bash, Kotlin (incl. `.kts`), and Lua.
- `--include-public` flag: also report public/exported symbols, which are now skipped by default
  per language (Rust public traits, TS/JS `export`s, Go capitalized names, Java/C# `public`
  members, PHP explicit-`public` members, Kotlin default-public declarations, Lua module-field
  functions).
- Extensionless scripts are detected via shebang sniffing (`#!/usr/bin/env python`, `bash`,
  `ruby`, `node`, `lua`, `php`, including `env -S` and `VAR=value` forms).
- Per-language test-file conventions (test files contribute references but their definitions are
  never reported): `_test.go`, `.test.`/`.spec.`/`__tests__/`, `src/test/`, `*Test.java`,
  `*Tests.cs`, `_spec.rb`, `Test.php`, `Test.kt`, `_spec.lua`, etc. `.d.ts` files are treated as
  reference-only.
- Per-language skip heuristics: Go `main`/`init`/`Test*`/`Benchmark*`/`Fuzz*`/`Example*`;
  Java/Kotlin/C# test-framework annotations and `@Override`/`override` methods; Ruby
  `initialize`/`method_missing`; PHP `__*` magic methods; `_`-prefixed names in TS/JS/Lua.
- Ruby dynamic-dispatch softening: symbols (`:name`, `key:`) and string arguments to
  `send`/`public_send`/`define_method`/`method`/`respond_to?` count as references.
- Bash bare argument words count as references (e.g. `trap cleanup EXIT`).

### Changed

- File discovery scans all supported extensions plus shebang scripts (previously only `.rs` and
  `.py`).
- Output kind labels extended for new node types: `interface`, `type`, `record`, `annotation`,
  `prop`, and broader mappings to `fn`/`class`/`var`/`const`.

### Fixed

- Same-name definitions no longer mask each other as "referenced" — each occurrence of a
  never-referenced name is now correctly reported.
