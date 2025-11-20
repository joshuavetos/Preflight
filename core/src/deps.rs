use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(root).map_err(|e| format!("Failed to read {}: {e}", root.display()))?
    {
        let entry = entry.map_err(|e| format!("Dir entry error: {e}"))?;
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

fn module_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

pub fn run() -> Result<(), String> {
    let root = PathBuf::from("core/src");
    let use_re = Regex::new(r"^use\\s+crate::([a-zA-Z0-9_]+)").unwrap();
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for file in rust_files(&root)? {
        let module = match module_name(&file) {
            Some(name) => name,
            None => continue,
        };
        let contents = fs::read_to_string(&file)
            .map_err(|e| format!("Failed to read {}: {e}", file.display()))?;
        let imports = graph.entry(module.clone()).or_default();
        for line in contents.lines() {
            if let Some(caps) = use_re.captures(line.trim_start()) {
                if let Some(dep) = caps.get(1) {
                    let target = dep.as_str();
                    if target != module {
                        imports.insert(target.to_string());
                    }
                }
            }
        }
    }

    for (module, deps) in graph {
        let deps_list = if deps.is_empty() {
            String::from("(no imports)")
        } else {
            deps.into_iter().collect::<Vec<_>>().join(", ")
        };
        println!("{} -> {}", module, deps_list);
    }

    Ok(())
}
