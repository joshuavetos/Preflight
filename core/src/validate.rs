use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct Violation {
    pub file: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    status: &'static str,
    violations: Vec<Violation>,
}

pub fn validate(json_output: bool) -> i32 {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

    let mut violations = match scan_files(&src_root, &manifest) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Validation failed: {}", err);
            return 1;
        }
    };

    let has_violations = !violations.is_empty();

    if json_output {
        let status = if has_violations { "violation" } else { "ok" };
        let report = ValidationReport { status, violations };
        match serde_json::to_string_pretty(&report) {
            Ok(rendered) => println!("{}", rendered),
            Err(err) => {
                eprintln!("Failed to render JSON: {}", err);
                return 1;
            }
        }
    } else if !has_violations {
        println!("Architecture validation passed.");
    } else {
        println!(
            "Architecture validation found {} violation(s):",
            violations.len()
        );
        for violation in &violations {
            println!("- {}: {}", violation.file, violation.message);
        }
    }

    if has_violations {
        1
    } else {
        0
    }
}

fn scan_files(src_root: &Path, manifest: &Path) -> Result<Vec<Violation>, String> {
    let mut files = Vec::new();
    collect_rs_files(src_root, &mut files)?;

    let mut violations = Vec::new();
    for file in files {
        violations.extend(validate_file(src_root, &file)?);
    }

    violations.extend(find_unused_imports(manifest)?);
    Ok(violations)
}

fn collect_rs_files(dir: &Path, acc: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?
    {
        let entry =
            entry.map_err(|e| format!("Failed to read entry in {}: {}", dir.display(), e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, acc)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            acc.push(path);
        }
    }
    Ok(())
}

fn validate_file(base: &Path, path: &Path) -> Result<Vec<Violation>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let relative = relative_path(base, path);
    let mut violations = Vec::new();

    if content.trim().is_empty() {
        violations.push(Violation {
            file: relative.clone(),
            message: "Empty module".to_string(),
        });
    }

    let imports: Vec<String> = content
        .lines()
        .map(str::trim_end)
        .filter(|line| is_use_declaration(line))
        .map(|line| line.trim().to_string())
        .collect();
    if !is_sorted(&imports) {
        violations.push(Violation {
            file: relative.clone(),
            message: "Imports not sorted lexicographically".to_string(),
        });
    }

    let mut modules = Vec::new();
    for line in content.lines() {
        if let Some(name) = module_name(line) {
            modules.push(name);
        }
    }
    if !modules.is_empty() {
        if !is_sorted(&modules) {
            violations.push(Violation {
                file: relative.clone(),
                message: "Module declarations not sorted".to_string(),
            });
        }

        let mut seen = HashSet::new();
        for module in modules {
            if !seen.insert(module.clone()) {
                violations.push(Violation {
                    file: relative.clone(),
                    message: format!("Duplicate module declaration: {}", module),
                });
            }
        }
    }

    Ok(violations)
}

fn relative_path(base: &Path, path: &Path) -> String {
    if let Ok(stripped) = path.strip_prefix(base) {
        if let Some(s) = stripped.to_str() {
            if let Some(base_name) = base.file_name().and_then(|b| b.to_str()) {
                return format!("{}/{}", base_name, s.replace('\\', "/"));
            }
            return s.replace('\\', "/");
        }
    }
    path.display().to_string()
}

fn is_use_declaration(line: &str) -> bool {
    let trimmed = line.trim_start();
    (trimmed.starts_with("use ") || trimmed.starts_with("pub use ")) && trimmed == line
}

fn module_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let prefix = if let Some(rest) = trimmed.strip_prefix("pub(crate) mod ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("pub mod ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("mod ") {
        rest
    } else {
        return None;
    };

    let end = prefix
        .find(|c: char| c == ';' || c == '{' || c.is_whitespace())
        .unwrap_or(prefix.len());
    let name = &prefix[..end];
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn is_sorted(items: &[String]) -> bool {
    items.windows(2).all(|pair| pair[0] <= pair[1])
}

fn find_unused_imports(manifest: &Path) -> Result<Vec<Violation>, String> {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(manifest)
        .env("RUSTFLAGS", "-Dwarnings")
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    let mut violations = Vec::new();
    let combined: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .chain(String::from_utf8_lossy(&output.stderr).lines())
        .map(|s| s.to_string())
        .collect();

    for window in combined.windows(2) {
        if let [first, second] = window {
            if first.contains("unused import") {
                let file = second.trim().trim_start_matches("-->").trim();
                let mut path = file.to_string();
                if let Some(base) = manifest.parent() {
                    if let Ok(stripped) = Path::new(&path).strip_prefix(base) {
                        if let Some(s) = stripped.to_str() {
                            path = s.replace('\\', "/");
                        }
                    }
                }
                violations.push(Violation {
                    file: path,
                    message: first.trim().to_string(),
                });
            }
        }
    }

    if output.status.success() || !violations.is_empty() {
        Ok(violations)
    } else {
        Err("cargo check failed for reasons unrelated to unused imports".to_string())
    }
}
