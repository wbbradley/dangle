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

More tree-sitter language support is planned for future releases.

## Reference Detection

Dangle recognizes references in:

- Direct identifier usage (function calls, type annotations, etc.)
- Method calls (field identifiers)
- String literals in Rust attributes (e.g., `#[serde(default = "my_default_fn")]`)
  - Paths like `"module::func_name"` extract the leaf segment as the reference

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
- Public/exported symbols, per language (pass `--include-public` to report them):
  - Rust: public traits (they may be implemented by downstream crates)
  - TypeScript/JavaScript: anything inside an `export` statement
  - Go: capitalized (exported) names
  - Java/C#: declarations with the `public` modifier
- Definitions marked with `nodangle` in a comment on the same line

Test files contribute references but their definitions are never reported. What counts as a
test file is per-language:

- Rust/Python: path contains `test_` or `/tests/`
- TypeScript/JavaScript: filename contains `.test.` or `.spec.`, or path contains `__tests__/`
- TypeScript: `.d.ts` ambient declaration files are also treated as reference-only
- Go: filename ends with `_test.go`
- Java: path contains `/src/test/`, or filename matches `Test*.java` / `*Test.java`
- C#: filename ends with `Test.cs` or `Tests.cs`, or path contains `/tests/`

## License

MIT
