mod analysis;
mod languages;
mod symbols;

use std::{collections::HashMap, path::Path, process::Command};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(name = "dangle")]
#[command(about = "Find dead code candidates in Rust and Python projects")]
struct Args {
    #[arg(short, long, help = "Show all definitions found")]
    verbose: bool,
}

fn get_git_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files"])
        .output()
        .context("Failed to run git ls-files")?;

    let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;

    Ok(stdout
        .lines()
        .filter(|line| {
            let path = Path::new(line);
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                (ext_str == "rs" || ext_str == "py") && !line.contains("test_")
            } else {
                false
            }
        })
        .map(String::from)
        .collect())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let files = get_git_files()?;

    let mut all_definitions = Vec::new();
    let mut ref_counts: HashMap<String, usize> = HashMap::new();

    for file_path in &files {
        let path = Path::new(file_path);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let lang = match languages::get_language_for_extension(ext) {
            Some(l) => l,
            None => continue,
        };

        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: Could not read {}: {}", file_path, e);
                continue;
            }
        };

        match symbols::extract_definitions(path, &source, lang) {
            Ok(defs) => {
                if args.verbose {
                    for def in &defs {
                        eprintln!(
                            "Found definition: {} in {}:{} ({})",
                            def.name, def.file, def.line, def.kind
                        );
                    }
                }
                all_definitions.extend(defs);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not parse definitions in {}: {}",
                    file_path, e
                );
            }
        }

        match symbols::extract_references(&source, lang) {
            Ok(refs) => {
                for r in refs {
                    *ref_counts.entry(r.name).or_insert(0) += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not parse references in {}: {}",
                    file_path, e
                );
            }
        }
    }

    let result = analysis::find_dead_code(all_definitions, &ref_counts);

    let mut dead_sorted: Vec<_> = result.dead_code.iter().collect();
    dead_sorted.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)));

    for def in dead_sorted {
        let kind_abbrev = match def.kind.as_str() {
            "function_item" | "function_definition" => "fn",
            "struct_item" => "struct",
            "enum_item" => "enum",
            "const_item" => "const",
            "static_item" => "static",
            "mod_item" => "mod",
            "class_definition" => "class",
            "assignment" => "var",
            _ => &def.kind,
        };
        println!("{}:{}: {} {}", def.file, def.line, kind_abbrev, def.name);
    }

    Ok(())
}
