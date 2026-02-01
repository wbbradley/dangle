use std::collections::HashMap;

use crate::symbols::Definition;

pub struct AnalysisResult {
    pub dead_code: Vec<Definition>,
}

pub fn find_dead_code(
    definitions: Vec<Definition>,
    ref_counts: &HashMap<String, usize>,
) -> AnalysisResult {
    let dead_code: Vec<Definition> = definitions
        .into_iter()
        .filter(|def| {
            if def.has_nodangle {
                return false;
            }
            let count = ref_counts.get(&def.name).unwrap_or(&0);
            *count <= 1
        })
        .collect();

    AnalysisResult { dead_code }
}
