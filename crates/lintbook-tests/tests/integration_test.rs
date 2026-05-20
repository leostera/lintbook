use insta::assert_json_snapshot;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn init_git(path: &Path) {
    Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .expect("Failed to init git repo");
}

fn run_lintbook(project_dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(workspace_root().join("Cargo.toml"))
        .arg("-p")
        .arg("lintbook-cli")
        .arg("--bin")
        .arg("lintbook")
        .arg("--");
    command.args(args);
    command
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute lintbook")
}

fn setup_generated_rule_project() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    init_git(temp_dir.path());

    fs::write(
        temp_dir.path().join("clean.rs"),
        "fn main() { println!(\"ok\"); }\n",
    )
    .unwrap();
    fs::write(temp_dir.path().join("bad.rs"), "fn main() { dbg!(1); }\n").unwrap();
    fs::write(
        temp_dir.path().join("lintbook.toml"),
        r#"[lintbook]
languages = ["rust"]
autofix = false

[lints]
"#,
    )
    .unwrap();

    let rules_dir = temp_dir.path().join(".lintbook").join("rules");
    let gen_dir = temp_dir.path().join(".lintbook").join("gen");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::create_dir_all(&gen_dir).unwrap();
    fs::write(
        rules_dir.join("no-dbg.md"),
        r#"---
id: rust.no-dbg
lang: rust
---

We don't want dbg! macro calls in production code.
"#,
    )
    .unwrap();
    fs::write(
        gen_dir.join("no-dbg.df"),
        r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")"#,
    )
    .unwrap();

    let compile = run_lintbook(temp_dir.path(), &["compile"]);
    assert!(
        compile.status.success(),
        "compile failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    temp_dir
}

fn run_lintbook_check(project_dir: &Path, files: &[&str]) -> Value {
    let mut args = vec!["check", "--output", "json"];
    args.extend(files.iter().copied());
    let output = run_lintbook(project_dir, &args);

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "unexpected lintbook exit: {:?}",
        output.status
    );
    assert!(
        output.stderr.is_empty(),
        "lintbook failed with error: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let mut json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("Failed to parse JSON: {error}. Output was: {stdout}"));

    if let Some(files) = json.get_mut("files").and_then(|value| value.as_array_mut()) {
        for file in &mut *files {
            if let Some(path) = file.get_mut("file_path").and_then(|value| value.as_str()) {
                let relative_path = Path::new(path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                *file.get_mut("file_path").unwrap() = Value::String(relative_path);
            }
            if let Some(duration) = file.get_mut("duration") {
                *duration = Value::from(0.0);
            }
        }
        files.sort_by(|left, right| {
            let left = left
                .get("file_path")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let right = right
                .get("file_path")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            left.cmp(right)
        });
    }

    if let Some(stats) = json
        .get_mut("statistics")
        .and_then(|value| value.as_object_mut())
    {
        if let Some(duration) = stats.get_mut("total_duration_ms") {
            *duration = Value::from(0.0);
        }
    }

    json
}

#[test]
fn setup_creates_lintbook_layout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    init_git(temp_dir.path());
    fs::write(temp_dir.path().join("main.rs"), "fn main() {}\n").unwrap();

    let output = run_lintbook(temp_dir.path(), &["setup"]);
    assert!(
        output.status.success(),
        "setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(temp_dir.path().join("lintbook.toml").exists());
    assert!(temp_dir.path().join(".lintbook/rules/template.md").exists());
    assert!(!temp_dir.path().join(".lintbook/gen/template.df").exists());
    assert!(temp_dir.path().join(".lintbook/gen/.gitkeep").exists());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Manual MCP configuration"), "{stdout}");
    assert!(stdout.contains(r#""args": ["mcp"]"#), "{stdout}");
}

#[test]
fn rust_clean_file_has_no_generated_violations() {
    let temp_dir = setup_generated_rule_project();
    let output = run_lintbook_check(temp_dir.path(), &["clean.rs"]);

    assert_json_snapshot!(output);
}

#[test]
fn rust_generated_rule_reports_violation() {
    let temp_dir = setup_generated_rule_project();
    let output = run_lintbook_check(temp_dir.path(), &["bad.rs"]);

    assert_json_snapshot!(output);
}

#[test]
fn check_accepts_directory_target() {
    let temp_dir = setup_generated_rule_project();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("nested_clean.rs"), "fn main() {}\n").unwrap();
    fs::write(src_dir.join("nested_bad.rs"), "fn main() { dbg!(1); }\n").unwrap();

    let output = run_lintbook_check(temp_dir.path(), &["src"]);
    let files = output["files"].as_array().unwrap();
    let file_names = files
        .iter()
        .map(|file| file["file_path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(output["statistics"]["total_files"], 2);
    assert!(file_names.contains(&"nested_clean.rs"));
    assert!(file_names.contains(&"nested_bad.rs"));
    assert_eq!(output["statistics"]["total_violations"], 1);
}

#[test]
fn check_accepts_glob_target() {
    let temp_dir = setup_generated_rule_project();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("glob_clean.rs"), "fn main() {}\n").unwrap();
    fs::write(src_dir.join("glob_bad.rs"), "fn main() { dbg!(1); }\n").unwrap();
    fs::write(src_dir.join("ignored.txt"), "dbg!(1)\n").unwrap();

    let output = run_lintbook_check(temp_dir.path(), &["src/glob_*.rs"]);
    let files = output["files"].as_array().unwrap();
    let file_names = files
        .iter()
        .map(|file| file["file_path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(output["statistics"]["total_files"], 2);
    assert!(file_names.contains(&"glob_clean.rs"));
    assert!(file_names.contains(&"glob_bad.rs"));
    assert_eq!(output["statistics"]["total_violations"], 1);
}

#[test]
fn check_accepts_recursive_glob_target() {
    let temp_dir = setup_generated_rule_project();
    let nested_dir = temp_dir.path().join("src").join("nested");
    fs::create_dir_all(&nested_dir).unwrap();
    fs::write(
        nested_dir.join("recursive_bad.rs"),
        "fn main() { dbg!(1); }\n",
    )
    .unwrap();

    let output = run_lintbook_check(temp_dir.path(), &["src/**/*.rs"]);
    assert_eq!(output["statistics"]["total_files"], 1);
    assert_eq!(output["statistics"]["total_violations"], 1);
    assert_eq!(output["files"][0]["file_path"], "recursive_bad.rs");
}

#[test]
fn check_accepts_mixed_directory_and_glob_targets() {
    let temp_dir = setup_generated_rule_project();
    let src_dir = temp_dir.path().join("src");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    fs::write(src_dir.join("dir_bad.rs"), "fn main() { dbg!(1); }\n").unwrap();
    fs::write(tests_dir.join("glob_bad.rs"), "fn main() { dbg!(1); }\n").unwrap();

    let output = run_lintbook_check(temp_dir.path(), &["src", "tests/*.rs"]);
    let files = output["files"].as_array().unwrap();
    let file_names = files
        .iter()
        .map(|file| file["file_path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(output["statistics"]["total_files"], 2);
    assert!(file_names.contains(&"dir_bad.rs"));
    assert!(file_names.contains(&"glob_bad.rs"));
    assert_eq!(output["statistics"]["total_violations"], 2);
}

#[test]
fn check_targets_are_relative_to_current_directory() {
    let temp_dir = setup_generated_rule_project();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("local_bad.rs"), "fn main() { dbg!(1); }\n").unwrap();

    let output = run_lintbook_check(&src_dir, &["local_bad.rs"]);
    assert_eq!(output["statistics"]["total_files"], 1);
    assert_eq!(output["statistics"]["total_violations"], 1);
    assert_eq!(output["files"][0]["file_path"], "local_bad.rs");
}

#[test]
fn check_json_flag_streams_json_lines() {
    let temp_dir = setup_generated_rule_project();
    let output = run_lintbook(temp_dir.path(), &["check", "--json", "bad.rs"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stderr.is_empty(),
        "lintbook failed with error: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2, "expected file event plus summary: {stdout}");

    let file_event: Value = serde_json::from_str(lines[0]).unwrap();
    let summary_event: Value = serde_json::from_str(lines[1]).unwrap();

    assert_eq!(file_event["type"], "file");
    assert_eq!(
        file_event["file"]["violations"][0]["lint_id"],
        "rust.no-dbg"
    );
    assert_eq!(summary_event["type"], "summary");
    assert_eq!(summary_event["statistics"]["total_files"], 1);
    assert_eq!(summary_event["statistics"]["total_violations"], 1);
}

#[test]
fn compile_fails_when_generated_query_is_missing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    init_git(temp_dir.path());

    let rules_dir = temp_dir.path().join(".lintbook").join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::create_dir_all(temp_dir.path().join(".lintbook").join("gen")).unwrap();
    fs::write(
        rules_dir.join("no-dbg.md"),
        r#"---
id: rust.no-dbg
lang: rust
---

We don't want dbg! macro calls in production code.
"#,
    )
    .unwrap();

    let output = run_lintbook(temp_dir.path(), &["compile"]);
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Incomplete rule(s):"), "{stdout}");
    assert!(stdout.contains(".lintbook/gen/no-dbg.df"), "{stdout}");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("lintbook compile --agent codex"),
        "{stderr}"
    );
}

#[test]
fn check_fails_when_generated_rules_are_stale() {
    let temp_dir = setup_generated_rule_project();
    fs::write(
        temp_dir.path().join(".lintbook/gen/no-dbg.df"),
        r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "println!")"#,
    )
    .unwrap();

    let output = run_lintbook(temp_dir.path(), &["check", "--output", "json", "bad.rs"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Run `lintbook compile`"), "{stderr}");
}
