use insta::assert_json_snapshot;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn setup_test_project(fixture_dir: &str) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.join("fixtures").join(fixture_dir);

    // Initialize git repo
    Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Copy all files from fixture to temp dir
    for entry in fs::read_dir(&fixture_path).expect("Failed to read fixture dir") {
        let entry = entry.expect("Failed to read entry");
        let dest = temp_dir.path().join(entry.file_name());
        fs::copy(entry.path(), dest).expect("Failed to copy fixture file");
    }

    // Create a treelint.toml in the temp dir
    let config = r#"[treelint]
languages = ["python"]
autofix = false

[lints]
"#;
    fs::write(temp_dir.path().join("treelint.toml"), config).expect("Failed to write config");

    temp_dir
}

fn run_treelint_check(project_dir: &Path) -> Value {
    // Build the treelint binary path
    let treelint_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("treelint");

    let output = Command::new(&treelint_path)
        .arg("check")
        .arg("--output")
        .arg("json")
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute treelint");

    // treelint exits with status 1 if there are violations, which is expected
    // Only panic if stderr is not empty (indicating an actual error)
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("treelint failed with error: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");

    if stdout.is_empty() {
        panic!("treelint produced empty output");
    }

    // Parse JSON and normalize file paths
    let mut json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "Failed to parse JSON output: {}. Output was: '{}'",
            e, stdout
        )
    });

    // Normalize file paths to be relative and durations to 0
    if let Some(files) = json.get_mut("files").and_then(|v| v.as_array_mut()) {
        for file in files {
            if let Some(path) = file.get_mut("file_path").and_then(|v| v.as_str()) {
                let relative_path = Path::new(path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                *file.get_mut("file_path").unwrap() = Value::String(relative_path);
            }
            // Normalize duration to 0 to avoid flaky tests
            if let Some(duration) = file.get_mut("duration") {
                *duration = Value::from(0.0);
            }
        }
    }

    // Normalize total duration in statistics
    if let Some(stats) = json.get_mut("statistics").and_then(|v| v.as_object_mut()) {
        if let Some(duration) = stats.get_mut("total_duration_ms") {
            *duration = Value::from(0.0);
        }
    }

    json
}

#[test]
fn test_python_clean_file() {
    let temp_dir = setup_test_project("python");
    let output = run_treelint_check(temp_dir.path());

    // Filter to only show clean_example.py results
    let mut filtered_output = output.clone();
    if let Some(files) = filtered_output
        .get_mut("files")
        .and_then(|v| v.as_array_mut())
    {
        files.retain(|f| {
            f.get("file_path")
                .and_then(|v| v.as_str())
                .map(|s| s == "clean_example.py")
                .unwrap_or(false)
        });
    }

    assert_json_snapshot!(filtered_output);
}

#[test]
fn test_python_file_with_violations() {
    let temp_dir = setup_test_project("python");
    let output = run_treelint_check(temp_dir.path());

    // Filter to only show test_example.py results
    let mut filtered_output = output.clone();
    if let Some(files) = filtered_output
        .get_mut("files")
        .and_then(|v| v.as_array_mut())
    {
        files.retain(|f| {
            f.get("file_path")
                .and_then(|v| v.as_str())
                .map(|s| s == "test_example.py")
                .unwrap_or(false)
        });
    }

    assert_json_snapshot!(filtered_output);
}

#[test]
fn test_python_all_files() {
    let temp_dir = setup_test_project("python");
    let output = run_treelint_check(temp_dir.path());

    // Sort files by name for consistent snapshots
    let mut sorted_output = output.clone();
    if let Some(files) = sorted_output
        .get_mut("files")
        .and_then(|v| v.as_array_mut())
    {
        files.sort_by(|a, b| {
            let path_a = a.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            let path_b = b.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            path_a.cmp(path_b)
        });
    }

    assert_json_snapshot!(sorted_output);
}
