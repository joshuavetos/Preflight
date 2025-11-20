use crate::analyze;
use crate::deps;
use crate::spec;
use crate::utils::json_envelope;
use serde_json::json;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;

fn add_file(zipper: &mut zip::ZipWriter<File>, path: &Path, base: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let options = FileOptions::default();
    zipper
        .start_file(relative.clone(), options)
        .map_err(|e| e.to_string())?;
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    zipper.write_all(&buf).map_err(|e| e.to_string())?;
    Ok(relative)
}

fn ensure_deps_output() -> Result<String, String> {
    let graph = deps::collect_graph()?;
    let payload = json_envelope("deps", "ok", json!({ "graph": graph.0 }));
    std::fs::create_dir_all(".preflight").map_err(|e| e.to_string())?;
    let path = ".preflight/deps.json";
    std::fs::write(path, serde_json::to_string_pretty(&payload).unwrap())
        .map_err(|e| e.to_string())?;
    Ok(path.to_string())
}

fn ensure_analysis_output() -> Option<String> {
    analyze::write_json().ok()
}

fn ensure_validate_env_output() -> Option<String> {
    spec::write_json().ok()
}

pub fn run(output: &str, json_output: bool) -> Result<(), String> {
    let mut included = Vec::new();
    let mut files = Vec::new();

    if Path::new(".preflight/scan.json").exists() {
        files.push(PathBuf::from(".preflight/scan.json"));
    }

    if let Ok(path) = ensure_deps_output() {
        files.push(PathBuf::from(path));
    }

    if let Some(path) = ensure_analysis_output() {
        files.push(PathBuf::from(path));
    }

    if let Some(path) = ensure_validate_env_output() {
        files.push(PathBuf::from(path));
    }

    for entry in WalkDir::new(".preflight/history").into_iter().flatten() {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    let base = Path::new(".");
    let file = File::create(output).map_err(|e| e.to_string())?;
    let mut zipper = zip::ZipWriter::new(file);

    for file in &files {
        let added = add_file(&mut zipper, file, base)?;
        included.push(added);
    }

    zipper.finish().map_err(|e| e.to_string())?;

    if json_output {
        let payload = json_envelope(
            "share",
            "ok",
            json!({
                "bundle_path": output,
                "included_files": included
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else {
        println!("Bundle written to {}", output);
        for f in &included {
            println!("- {}", f);
        }
    }

    Ok(())
}
