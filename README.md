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
- `-h, --help` - Print help

## How It Works

Dangle uses [tree-sitter](https://tree-sitter.github.io/) for accurate AST-based parsing. Unlike
regex-based approaches, it correctly ignores symbols that appear only in strings or comments.

The algorithm:

1. Discovers source files via `git ls-files`
2. Extracts definitions from non-test files using tree-sitter queries
3. Extracts all identifier references from all files (including tests)
4. Reports definitions that are referenced only once (the definition itself)

This means symbols used only in tests won't be flagged as dead code.

## Supported Languages

- Rust
- Python

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
- Functions inside Rust trait impls (e.g., `impl Drop`, `impl Iterator`, etc.)
- Private traits that are not implemented (public traits are assumed to be API)
- Definitions marked with `nodangle` in a comment on the same line

## License

MIT
