use std::collections::HashMap;

use crate::symbols::Definition;

pub struct AnalysisResult {
    pub dead_code: Vec<Definition>,
}

pub fn find_dead_code(
    definitions: Vec<Definition>,
    ref_counts: &HashMap<String, usize>,
) -> AnalysisResult {
    // Each definition's own name shows up in the reference counts, so a name defined
    // in N places (e.g. C# partial classes, same-name fns in different modules) needs
    // more than N references to be considered alive.
    let mut def_counts: HashMap<&str, usize> = HashMap::new();
    for def in &definitions {
        *def_counts.entry(def.name.as_str()).or_insert(0) += 1;
    }

    let dead_code: Vec<Definition> = definitions
        .iter()
        .filter(|def| {
            if def.has_nodangle {
                return false;
            }
            let count = ref_counts.get(&def.name).unwrap_or(&0);
            *count <= def_counts[def.name.as_str()]
        })
        .cloned()
        .collect();

    AnalysisResult { dead_code }
}
