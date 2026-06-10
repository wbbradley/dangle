# dangle

A dead code detector for multi-language projects. Finds symbols that are defined but never
referenced elsewhere in your codebase.

## Installation

```sh
cargo install dangle
```

## Usage

Run `dangle` in a git repository to find dead code candidates:

```sh
dangle
```

Output format:

```
path/to/file.rs:42:5: fn unused_function is not referenced
path/to/file.py:17:1: class UnusedClass is not referenced
```

### Options

- `-v, --verbose` - Show all definitions found
- `--include-public` - Also report public/exported symbols (skipped by default)
- `-h, --help` - Print help

## How It Works

Dangle uses [tree-sitter](https://tree-sitter.github.io/) for accurate AST-based parsing. Unlike
regex-based approaches, it correctly ignores symbols that appear only in strings or comments.

The algorithm:

1. Discovers source files via `git ls-files`
2. Extracts definitions from non-test files using tree-sitter queries
3. Extracts all identifier references from all files (including tests)
4. Reports definitions whose names are never referenced beyond the definitions themselves
   (multiple same-name definitions — e.g. C# partial classes — do not keep each other alive)

This means symbols used only in tests won't be flagged as dead code.

## Supported Languages

- Rust
- Python
- TypeScript / TSX
- JavaScript / JSX
- Go
- Java
- C#
- Ruby
- PHP
- Bash
- Kotlin
- Lua

More tree-sitter language support is planned for future releases.

## Reference Detection

Dangle recognizes references in:

- Direct identifier usage (function calls, type annotations, etc.)
- Method calls (field identifiers)
- String literals in Rust attributes (e.g., `#[serde(default = "my_default_fn")]`)
  - Paths like `"module::func_name"` extract the leaf segment as the reference
- Ruby symbols (`:method_name`, `key:` hash keys) and string arguments to
  `send`/`public_send`/`define_method`/`method`/`respond_to?`
- Bash command names and bare argument words (e.g. `trap cleanup EXIT`)

## Filters

Dangle automatically excludes:

- `main` functions
- `test_*` functions and `Test*` classes (Python)
- `#[test]` functions (Rust)
- `#[allow(unused)]` and `#[allow(dead_code)]` definitions (Rust)
- `__*` names (Python dunders, etc.)
- `_*` names (TypeScript/JavaScript intentionally-unused convention)
- Functions inside Rust trait impls (e.g., `impl Drop`, `impl Iterator`, etc.)
- Go: `main`, `init`, `_`, and `Test*`/`Benchmark*`/`Fuzz*`/`Example*` functions
- Java: `main`, `toString`, `equals`, `hashCode`, and methods annotated with `@Test`,
  `@ParameterizedTest`, `@RepeatedTest`, or `@Override` (overrides are invoked via the supertype)
- C#: `Main`, and methods attributed with `[Test]`, `[Fact]`, `[Theory]`, or `[TestMethod]`
- Ruby: `initialize`, `method_missing`, `respond_to_missing?`
- PHP: `__*` magic methods (`__construct`, `__get`, ...)
- Bash: `main`
- Kotlin: `main`, `@Test`/`@ParameterizedTest`-annotated functions, and `override` functions
  (overrides are invoked via the supertype)
- Lua: `_*` names (intentionally-unused convention, metamethods like `__index`)
- Public/exported symbols, per language (pass `--include-public` to report them):
  - Rust: public traits (they may be implemented by downstream crates)
  - TypeScript/JavaScript: anything inside an `export` statement
  - Go: capitalized (exported) names
  - Java/C#: declarations with the `public` modifier
  - PHP: class members with an explicit `public` modifier (no-modifier members and all
    top-level functions/classes stay reportable)
  - Kotlin: top-level and class-member declarations without a `private`/`internal`
    modifier (default visibility is public; function-local `val`/`var` stay reportable)
  - Lua: module-field functions (`function M.foo()`, `function M:bar()`,
    `M.foo = function()`) — Lua's export idiom; plain locals/globals stay reportable
- Definitions marked with `nodangle` in a comment on the same line

Test files contribute references but their definitions are never reported. What counts as a
test file is per-language:

- Rust/Python: path contains `test_` or `/tests/`
- TypeScript/JavaScript: filename contains `.test.` or `.spec.`, or path contains `__tests__/`
- TypeScript: `.d.ts` ambient declaration files are also treated as reference-only
- Go: filename ends with `_test.go`
- Java: path contains `/src/test/`, or filename matches `Test*.java` / `*Test.java`
- C#: filename ends with `Test.cs` or `Tests.cs`, or path contains `/tests/`
- Ruby: filename ends with `_spec.rb` or `_test.rb`, or path contains a `spec/` or
  `test/` directory
- PHP: filename ends with `Test.php`, or path contains a `tests/` directory
- Kotlin: path contains `src/test/`, or filename ends with `Test.kt`
- Lua: filename ends with `_spec.lua`, or path contains a `spec/` or `tests/` directory
  (busted convention)

## Caveats for Dynamic Languages

Name-based counting cannot see dynamic dispatch:

- Ruby: `send` with a computed name and `method_missing`-based dispatch are invisible,
  so methods invoked only that way may be falsely reported. Symbol literals and string
  arguments to `send`/`define_method` are counted to soften this.
- PHP: variable functions (`$f()`, `call_user_func($var)`) are invisible to name-based
  counting; functions invoked only that way may be falsely reported.
- Bash: quoted function names in arguments (`trap 'cleanup' EXIT`) are not counted —
  use the bare form (`trap cleanup EXIT`) or a `nodangle` comment.

## License

MIT
