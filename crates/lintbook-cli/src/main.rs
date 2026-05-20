use colored::*;
use lintbook_config::LintbookConfig;
use lintbook_core::{LintResult, LintStatus};
use lintbook_lang::Grammar;
use lintbook_scanner::Scanner;
use serde::Serialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

const CONFIG_FILE: &str = "lintbook.toml";
const LINTBOOK_DIR: &str = ".lintbook";
const RULES_DIR: &str = "rules";
const GEN_DIR: &str = "gen";

#[derive(Debug, Clone)]
enum OutputFormat {
    Human,
    Json,
    JsonStream,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!(
                "Invalid output format: {}. Expected 'human' or 'json'",
                s
            )),
        }
    }
}

#[derive(StructOpt)]
#[structopt(name = "lintbook")]
#[structopt(about = "Author and run fast tree-sitter lintbooks")]
enum Cli {
    #[structopt(about = "Set up lintbook in the current repository")]
    Setup,
    #[structopt(about = "Create a custom lintbook rule description")]
    Add {
        #[structopt(help = "Natural-language description of the check")]
        description: String,
    },
    #[structopt(about = "Compile .lintbook/rules Markdown plus .lintbook/gen Datafox queries")]
    Compile {
        #[structopt(
            long,
            help = "Generate missing queries with an agent first (currently: codex)"
        )]
        agent: Option<String>,
        #[structopt(
            last = true,
            multiple = true,
            allow_hyphen_values = true,
            help = "Extra arguments passed to the Codex CLI before `exec`"
        )]
        agent_args: Vec<String>,
    },
    #[structopt(about = "Check repository for lint violations")]
    Check {
        #[structopt(long, help = "Stream newline-delimited JSON events")]
        json: bool,
        #[structopt(long, default_value = "human", help = "Output format (human, json)")]
        output: OutputFormat,
        #[structopt(
            help = "Optional files, directories, or glob patterns to check (if not provided, scans entire repository)"
        )]
        files: Vec<String>,
    },
    #[structopt(about = "List all available built-in lint rules")]
    Lints {
        #[structopt(long, help = "Show only rules for specific language")]
        language: Option<String>,
        #[structopt(long, default_value = "human", help = "Output format (human, json)")]
        output: OutputFormat,
    },
    #[structopt(about = "Dump AST of a file or stdin input")]
    DumpAst {
        #[structopt(
            long,
            short,
            help = "Language to parse (e.g., python, rust, sql). Optional when file provided - inferred from extension"
        )]
        lang: Option<String>,
        #[structopt(help = "File to parse (if not provided, reads from stdin)")]
        file: Option<String>,
    },
    #[structopt(about = "Run the lintbook MCP server over stdio")]
    Mcp,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::from_args() {
        Cli::Setup => setup_project()?,
        Cli::Add { description } => add_rule(&description)?,
        Cli::Compile { agent, agent_args } => compile_rules(agent, agent_args)?,
        Cli::Check {
            json,
            output,
            files,
        } => {
            let output = if json {
                OutputFormat::JsonStream
            } else {
                output
            };
            check_project(output, files).await?
        }
        Cli::Lints { language, output } => list_lints(language, output)?,
        Cli::DumpAst { lang, file } => dump_ast(lang, file)?,
        Cli::Mcp => lintbook_mcp::run_stdio_server().await?,
    }

    Ok(())
}

fn setup_project() -> anyhow::Result<()> {
    println!("Scanning repository for supported file types...");

    let current_dir = env::current_dir()?;
    let repo_root = Scanner::find_repo_root(&current_dir)?;
    println!("Found repository root: {}", repo_root.display());

    let grammars = Scanner::detect_grammars(&repo_root)?;
    if grammars.is_empty() {
        println!("No supported file types found in repository");
    } else {
        println!("Detected {} grammar(s):", grammars.len());
        for grammar in &grammars {
            println!("  - {}", grammar.name());
        }
    }

    let config_path = repo_root.join(CONFIG_FILE);
    if config_path.exists() {
        println!("Keeping existing {}", config_path.display());
    } else {
        let grammar_names = grammars.iter().map(|g| g.name()).collect::<Vec<_>>();
        LintbookConfig::new(grammar_names).write_to_file(&config_path)?;
        println!("Created {}", config_path.display());
    }

    create_lintbook_layout(&repo_root)?;
    maybe_offer_precommit_hook(&repo_root)?;
    print_mcp_instructions();

    println!("lintbook setup complete");
    Ok(())
}

fn create_lintbook_layout(repo_root: &Path) -> anyhow::Result<()> {
    let lintbook_dir = repo_root.join(LINTBOOK_DIR);
    let rules_dir = lintbook_dir.join(RULES_DIR);
    let gen_dir = lintbook_dir.join(GEN_DIR);

    fs::create_dir_all(&rules_dir)?;
    fs::create_dir_all(&gen_dir)?;
    write_if_missing(&gen_dir.join(".gitkeep"), "")?;
    write_if_missing(&rules_dir.join("template.md"), TEMPLATE_MD)?;

    println!("Prepared {}", lintbook_dir.display());
    Ok(())
}

fn maybe_offer_precommit_hook(repo_root: &Path) -> anyhow::Result<()> {
    if !io::stdin().is_terminal() {
        println!("Pre-commit hook not changed. Add this to .git/hooks/pre-commit if desired:");
        println!("{}", PRECOMMIT_SNIPPET.trim_end());
        return Ok(());
    }

    print!("Add a pre-commit hook that runs `lintbook check`? [y/N] ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
        println!("Pre-commit hook not changed.");
        return Ok(());
    }

    let hook_path = repo_root.join(".git").join("hooks").join("pre-commit");
    if hook_path.exists() {
        println!(
            "{} already exists. Add this snippet manually:",
            hook_path.display()
        );
        println!("{}", PRECOMMIT_SNIPPET.trim_end());
        return Ok(());
    }

    fs::write(&hook_path, PRECOMMIT_SNIPPET)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions)?;
    }
    println!("Installed {}", hook_path.display());
    Ok(())
}

fn print_mcp_instructions() {
    println!();
    println!("Manual MCP configuration:");
    println!(
        r#"{{
  "mcpServers": {{
    "lintbook": {{
      "command": "lintbook",
      "args": ["mcp"]
    }}
  }}
}}"#
    );
}

fn add_rule(description: &str) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let repo_root = Scanner::find_repo_root(&current_dir)?;
    let rules_dir = repo_root.join(LINTBOOK_DIR).join(RULES_DIR);
    if !rules_dir.exists() {
        anyhow::bail!("No .lintbook/rules directory found. Run `lintbook setup` first.");
    }

    let slug = slugify(description);
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let stem = format!("{}-{}", slug, timestamp);
    let id = format!("rust.{}", stem);
    let md_path = rules_dir.join(format!("{stem}.md"));

    let markdown = format!(
        r#"---
id: {}
lang: rust
---

{}
"#,
        id, description
    );

    fs::write(&md_path, markdown)?;

    println!("Created {}", md_path.display());
    println!("Generate .lintbook/gen/{stem}.df, then run `lintbook compile`.");
    Ok(())
}

fn compile_rules(agent: Option<String>, agent_args: Vec<String>) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let repo_root = Scanner::find_repo_root(&current_dir)?;

    if let Some(agent) = agent {
        run_compile_agent(&repo_root, &agent, &agent_args)?;
    }

    let report = lintbook_rules::compile_project(&repo_root)?;

    println!(
        "Compiled {} rule(s), skipped {} incomplete rule(s).",
        report.compiled.len(),
        report.skipped_incomplete.len()
    );
    for id in report.compiled {
        println!("  - {}", id);
    }
    if !report.skipped_incomplete.is_empty() {
        println!();
        println!("Incomplete rule(s):");
        for rule in &report.skipped_incomplete {
            println!(
                "  - {}: missing {}",
                rule.id,
                relative_display(&repo_root, &rule.query_path)
            );
        }
        anyhow::bail!(
            "{} incomplete rule(s). Generate the missing .df files or run `lintbook compile --agent codex`.",
            report.skipped_incomplete.len()
        );
    }

    Ok(())
}

fn run_compile_agent(repo_root: &Path, agent: &str, agent_args: &[String]) -> anyhow::Result<()> {
    if agent != "codex" {
        anyhow::bail!("Unsupported compile agent `{agent}`. Supported agents: codex");
    }

    let incomplete = lintbook_rules::incomplete_rules(repo_root)?;
    if incomplete.is_empty() {
        println!("No incomplete rules found for agent generation.");
        return Ok(());
    }

    println!(
        "Generating {} missing Datafox query file(s) with Codex...",
        incomplete.len()
    );

    let prompt = build_codex_compile_prompt(repo_root, &incomplete)?;
    let command_args = build_codex_exec_args(repo_root, agent_args);
    let mut child = Command::new("codex")
        .args(&command_args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| anyhow::anyhow!("Failed to start Codex agent: {error}"))?;

    child
        .stdin
        .take()
        .expect("Codex stdin is piped")
        .write_all(prompt.as_bytes())?;

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("Codex agent exited with status {status}");
    }

    Ok(())
}

fn build_codex_exec_args(repo_root: &Path, agent_args: &[String]) -> Vec<String> {
    let mut args = Vec::new();

    if agent_args.is_empty() {
        args.push("--ask-for-approval".to_string());
        args.push("never".to_string());
    } else {
        args.extend(agent_args.iter().cloned());
    }

    args.push("exec".to_string());
    args.push("--cd".to_string());
    args.push(repo_root.display().to_string());
    args.push("--sandbox".to_string());
    args.push("workspace-write".to_string());
    args.push("-".to_string());
    args
}

fn build_codex_compile_prompt(
    repo_root: &Path,
    incomplete: &[lintbook_rules::IncompleteRule],
) -> anyhow::Result<String> {
    let mut prompt = String::from(
        r#"You are generating lintbook Datafox query files.

For each incomplete rule below:
- Read the Markdown rule intent.
- Use the Datafox grammar, fact schema, examples, and testing workflow in the guide below.
- Write one Datafox query set to the requested .lintbook/gen/<stem>.df path.
- Use `Node` as the primary variable that identifies the violation node.
- Do not edit the .md files unless the rule intent is impossible to understand.
- Do not hand-edit .json compiled artifacts.
- Test the query with at least one positive example and one negative example before finishing.
- After writing .df files, run `cargo run --quiet -p lintbook-cli --bin lintbook -- compile` without `--agent`.

Authoring guide:
"#,
    );
    prompt.push_str(lintbook_rules::RULE_AUTHORING_GUIDE);
    prompt.push_str(
        r#"

"#,
    );

    for rule in incomplete {
        let markdown = fs::read_to_string(&rule.markdown_path)?;
        prompt.push_str("Rule:\n");
        prompt.push_str(&format!("- id: {}\n", rule.id));
        prompt.push_str(&format!(
            "- markdown: {}\n",
            relative_display(repo_root, &rule.markdown_path)
        ));
        prompt.push_str(&format!(
            "- write query: {}\n",
            relative_display(repo_root, &rule.query_path)
        ));
        prompt.push_str("Markdown contents:\n```markdown\n");
        prompt.push_str(&markdown);
        if !markdown.ends_with('\n') {
            prompt.push('\n');
        }
        prompt.push_str("```\n\n");
    }

    Ok(prompt)
}

fn relative_display(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

async fn check_project(output: OutputFormat, files: Vec<String>) -> anyhow::Result<()> {
    let scan_start = std::time::Instant::now();

    if matches!(output, OutputFormat::Human) {
        println!("Scanning repository for lint violations...");
    }

    let current_dir = env::current_dir()?;
    let repo_root = Scanner::find_repo_root(&current_dir)?;
    let config = read_config(&repo_root, &output)?;
    let active_rule_languages = lintbook_rules::active_rule_languages(&repo_root, &config)?;
    let evaluation_profile = generated_rule_evaluation_profile(&current_dir, &files);

    if !matches!(output, OutputFormat::Json) {
        return stream_check_results(
            output,
            &repo_root,
            &current_dir,
            &config,
            &files,
            &active_rule_languages,
            evaluation_profile,
            scan_start,
        );
    }

    let mut results = collect_lint_results(
        &repo_root,
        &current_dir,
        &config,
        &files,
        &active_rule_languages,
    )?;
    let generated_violations = lintbook_rules::run_generated_rules_with_profile(
        &repo_root,
        &config,
        &results,
        evaluation_profile,
    )
    .await?;
    for result in &mut results {
        if let Some(mut violations) = generated_violations.get(&result.file_path).cloned() {
            result.violations.append(&mut violations);
            result.status = LintStatus::Error;
        }
    }

    let llm_violations = run_llm_lints(&config, &repo_root, &results).await?;
    print_check_results(output, &repo_root, results, llm_violations, scan_start)
}

fn generated_rule_evaluation_profile(
    current_dir: &Path,
    files: &[String],
) -> lintbook_rules::GeneratedRuleEvaluationProfile {
    if files.len() == 1 && is_explicit_file_target(current_dir, &files[0]) {
        return lintbook_rules::GeneratedRuleEvaluationProfile::parallel();
    }

    lintbook_rules::GeneratedRuleEvaluationProfile::serial()
}

fn is_explicit_file_target(current_dir: &Path, target: &str) -> bool {
    if target
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'['))
    {
        return false;
    }

    let path = Path::new(target);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    path.is_file()
}

fn read_config(repo_root: &Path, output: &OutputFormat) -> anyhow::Result<LintbookConfig> {
    let config_path = repo_root.join(CONFIG_FILE);
    match LintbookConfig::read_from_file(&config_path) {
        Ok(config) => Ok(config),
        Err(_) => {
            match output {
                OutputFormat::Human => {
                    println!("No lintbook.toml found. Run `lintbook setup` first.")
                }
                OutputFormat::Json => {
                    eprintln!("Error: No lintbook.toml found. Run `lintbook setup` first.");
                }
                OutputFormat::JsonStream => {
                    eprintln!("Error: No lintbook.toml found. Run `lintbook setup` first.");
                }
            }
            anyhow::bail!("missing lintbook.toml")
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct CheckStatistics {
    total_duration_ms: f64,
    total_files: usize,
    files_scanned: usize,
    files_skipped: usize,
    files_with_violations: usize,
    total_violations: usize,
    llm_violations_count: usize,
}

#[derive(Debug, Default)]
struct CheckCounters {
    total_files: usize,
    files_scanned: usize,
    files_skipped: usize,
    files_with_violations: usize,
    total_violations: usize,
    llm_violations_count: usize,
}

impl CheckCounters {
    fn record_result(&mut self, result: &LintResult<Grammar>) {
        self.total_files += 1;
        match result.status {
            LintStatus::Ok | LintStatus::Error => self.files_scanned += 1,
            LintStatus::Skipped => self.files_skipped += 1,
        }
        if !result.violations.is_empty() {
            self.files_with_violations += 1;
        }
        self.total_violations += result.violations.len();
    }

    fn statistics(&self, scan_start: std::time::Instant) -> CheckStatistics {
        CheckStatistics {
            total_duration_ms: scan_start.elapsed().as_micros() as f64 / 1000.0,
            total_files: self.total_files,
            files_scanned: self.files_scanned,
            files_skipped: self.files_skipped,
            files_with_violations: self.files_with_violations,
            total_violations: self.total_violations,
            llm_violations_count: self.llm_violations_count,
        }
    }
}

#[derive(Serialize)]
struct JsonLineFileEvent<'a> {
    #[serde(rename = "type")]
    event_type: &'static str,
    file: &'a LintResult<Grammar>,
}

#[derive(Serialize)]
struct JsonLineSummaryEvent {
    #[serde(rename = "type")]
    event_type: &'static str,
    statistics: CheckStatistics,
}

fn stream_check_results(
    output: OutputFormat,
    repo_root: &Path,
    current_dir: &Path,
    config: &LintbookConfig,
    files: &[String],
    active_rule_languages: &HashSet<Grammar>,
    evaluation_profile: lintbook_rules::GeneratedRuleEvaluationProfile,
    scan_start: std::time::Instant,
) -> anyhow::Result<()> {
    let runner = Arc::new(lintbook_rules::GeneratedRuleRunner::new_with_profile(
        repo_root,
        config,
        evaluation_profile,
    )?);
    let counters = Arc::new(Mutex::new(CheckCounters::default()));
    let repo_root_for_handler = repo_root.to_path_buf();
    let output_for_handler = output.clone();
    let counters_for_handler = Arc::clone(&counters);
    let runner_for_handler = Arc::clone(&runner);

    stream_lint_results(
        repo_root,
        current_dir,
        config,
        files,
        active_rule_languages,
        move |mut result| {
            let mut generated = runner_for_handler.run_on_lint_result(&result)?;
            if !generated.is_empty() {
                result.violations.append(&mut generated);
                result.status = LintStatus::Error;
            }

            let mut counters = counters_for_handler.lock().unwrap();
            counters.record_result(&result);
            match output_for_handler {
                OutputFormat::Human => {
                    let mut stdout = io::stdout().lock();
                    print_human_file_result(&mut stdout, &repo_root_for_handler, &result)?;
                    stdout.flush()?;
                }
                OutputFormat::JsonStream => {
                    let mut stdout = io::stdout().lock();
                    serde_json::to_writer(
                        &mut stdout,
                        &JsonLineFileEvent {
                            event_type: "file",
                            file: &result,
                        },
                    )?;
                    writeln!(stdout)?;
                    stdout.flush()?;
                }
                OutputFormat::Json => {}
            }
            Ok(())
        },
    )?;

    let statistics = counters.lock().unwrap().statistics(scan_start);
    match output {
        OutputFormat::Human => {
            println!(
                "\nLintbooked project in {:.2}ms",
                statistics.total_duration_ms
            );
        }
        OutputFormat::JsonStream => {
            let mut stdout = io::stdout().lock();
            serde_json::to_writer(
                &mut stdout,
                &JsonLineSummaryEvent {
                    event_type: "summary",
                    statistics: statistics.clone(),
                },
            )?;
            writeln!(stdout)?;
            stdout.flush()?;
        }
        OutputFormat::Json => {}
    }

    if statistics.total_violations > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn stream_lint_results<F>(
    repo_root: &Path,
    current_dir: &Path,
    config: &LintbookConfig,
    files: &[String],
    active_rule_languages: &HashSet<Grammar>,
    handler: F,
) -> anyhow::Result<()>
where
    F: Fn(LintResult<Grammar>) -> anyhow::Result<()> + Send + Sync + 'static,
{
    if files.is_empty() {
        return Scanner::scan_and_lint_files_with(
            repo_root,
            config,
            active_rule_languages,
            handler,
        );
    }

    Scanner::scan_and_lint_targets_with(
        repo_root,
        current_dir,
        files,
        config,
        active_rule_languages,
        handler,
    )
}

fn print_human_file_result(
    stdout: &mut impl Write,
    repo_root: &Path,
    result: &LintResult<Grammar>,
) -> anyhow::Result<()> {
    let file_path = result
        .file_path
        .strip_prefix(repo_root)
        .unwrap_or(&result.file_path)
        .display();
    let duration_ms = result.duration.as_micros() as f64 / 1000.0;

    match result.status {
        LintStatus::Ok => {
            writeln!(
                stdout,
                "{} {} {:.2}ms",
                file_path,
                "OK".green(),
                duration_ms
            )?;
        }
        LintStatus::Skipped => {
            writeln!(
                stdout,
                "{}",
                format!("{} SKIP {:.2}ms", file_path, duration_ms).dimmed()
            )?;
        }
        LintStatus::Error => {
            writeln!(stdout, "{} {} {:.2}ms", file_path, "ERR".red(), duration_ms)?;
            for violation in &result.violations {
                let filename = result
                    .file_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| file_path.to_string());

                writeln!(
                    stdout,
                    "  {}:{}:{} {} ({}) - {}",
                    filename.dimmed(),
                    violation.line.to_string().yellow(),
                    violation.column.to_string().yellow(),
                    violation.lint_id.bright_cyan(),
                    violation.lint_name.cyan(),
                    violation.message
                )?;
            }
        }
    }

    Ok(())
}

fn collect_lint_results(
    repo_root: &Path,
    current_dir: &Path,
    config: &LintbookConfig,
    files: &[String],
    active_rule_languages: &HashSet<Grammar>,
) -> anyhow::Result<Vec<LintResult<Grammar>>> {
    if files.is_empty() {
        return Scanner::scan_and_lint_files(repo_root, config, active_rule_languages);
    }

    Scanner::scan_and_lint_targets(repo_root, current_dir, files, config, active_rule_languages)
}

fn print_check_results(
    output: OutputFormat,
    repo_root: &Path,
    results: Vec<LintResult<Grammar>>,
    llm_violations: Vec<lintbook_core::LintViolation>,
    scan_start: std::time::Instant,
) -> anyhow::Result<()> {
    match output {
        OutputFormat::Json | OutputFormat::JsonStream => {
            let total_duration = scan_start.elapsed();
            let total_violations: usize =
                results.iter().map(|r| r.violations.len()).sum::<usize>() + llm_violations.len();
            let files_with_violations = results.iter().filter(|r| !r.violations.is_empty()).count();
            let files_scanned = results
                .iter()
                .filter(|r| matches!(r.status, LintStatus::Ok | LintStatus::Error))
                .count();
            let files_skipped = results
                .iter()
                .filter(|r| matches!(r.status, LintStatus::Skipped))
                .count();

            let output = serde_json::json!({
                "files": results,
                "llm_violations": llm_violations,
                "statistics": {
                    "total_duration_ms": total_duration.as_micros() as f64 / 1000.0,
                    "total_files": results.len(),
                    "files_scanned": files_scanned,
                    "files_skipped": files_skipped,
                    "files_with_violations": files_with_violations,
                    "total_violations": total_violations,
                    "llm_violations_count": llm_violations.len(),
                }
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
            if total_violations > 0 {
                std::process::exit(1);
            }
        }
        OutputFormat::Human => {
            let mut total_violations = 0;

            for result in &results {
                let file_path = result
                    .file_path
                    .strip_prefix(repo_root)
                    .unwrap_or(&result.file_path)
                    .display();
                let duration_ms = result.duration.as_micros() as f64 / 1000.0;

                match result.status {
                    LintStatus::Ok => {
                        println!("{} {} {:.2}ms", file_path, "OK".green(), duration_ms);
                    }
                    LintStatus::Skipped => {
                        println!(
                            "{}",
                            format!("{} SKIP {:.2}ms", file_path, duration_ms).dimmed()
                        );
                    }
                    LintStatus::Error => {
                        println!("{} {} {:.2}ms", file_path, "ERR".red(), duration_ms);
                        for violation in &result.violations {
                            let filename = result
                                .file_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| file_path.to_string());

                            println!(
                                "  {}:{}:{} {} ({}) - {}",
                                filename.dimmed(),
                                violation.line.to_string().yellow(),
                                violation.column.to_string().yellow(),
                                violation.lint_id.bright_cyan(),
                                violation.lint_name.cyan(),
                                violation.message
                            );
                            total_violations += 1;
                        }
                    }
                }
            }

            for llm_violation in &llm_violations {
                println!(
                    "  LLM:{}:{} {} (LLM) - {}",
                    llm_violation.line.to_string().yellow(),
                    llm_violation.column.to_string().yellow(),
                    llm_violation.lint_name.bright_magenta(),
                    llm_violation.message
                );
                total_violations += 1;
            }

            let total_ms = scan_start.elapsed().as_micros() as f64 / 1000.0;
            println!("\nLintbooked project in {:.2}ms", total_ms);

            if total_violations > 0 {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn list_lints(language: Option<String>, output: OutputFormat) -> anyhow::Result<()> {
    let builtin_lints = lintbook_rules::builtin_rule_infos()?;
    let mut all_lints = Vec::new();

    if let Some(lang) = language {
        let language = lang.to_lowercase();
        let grammar = lintbook_lang::get_supported_grammars()
            .into_iter()
            .find(|g| g.name() == language);

        if let Some(grammar) = grammar {
            all_lints.extend(
                builtin_lints
                    .into_iter()
                    .filter(|lint| lint.language == grammar.name()),
            );
        } else {
            match output {
                OutputFormat::Human => {
                    println!(
                        "Unknown language: {}. Use `lintbook lints` to see all supported languages.",
                        lang
                    );
                }
                OutputFormat::Json | OutputFormat::JsonStream => {
                    eprintln!("Error: Unknown language: {}", lang);
                }
            }
            return Ok(());
        }
    } else {
        all_lints = builtin_lints;
    }

    match output {
        OutputFormat::Json | OutputFormat::JsonStream => {
            let mut languages = std::collections::HashMap::new();
            for lint in all_lints {
                let lang_entry = languages
                    .entry(lint.language.clone())
                    .or_insert_with(Vec::new);
                lang_entry.push(serde_json::json!({
                    "id": lint.id,
                    "name": lint.name,
                    "description": lint.description,
                    "explanation": lint.description
                }));
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({ "languages": languages }))?
            );
        }
        OutputFormat::Human => {
            if all_lints.is_empty() {
                println!("No lint rules available.");
                return Ok(());
            }

            println!("Available built-in lint rules\n");
            let mut language_lints: std::collections::HashMap<String, Vec<_>> =
                std::collections::HashMap::new();
            for lint in &all_lints {
                language_lints
                    .entry(lint.language.clone())
                    .or_default()
                    .push(lint);
            }

            let mut languages = language_lints.keys().cloned().collect::<Vec<_>>();
            languages.sort();
            for lang in languages {
                let lints = &language_lints[&lang];
                println!("{} rules ({}):", lang.to_uppercase(), lints.len());
                for lint in lints {
                    println!(
                        "  {} {} - {}",
                        lint.id.bright_cyan(),
                        lint.name.cyan(),
                        lint.description
                    );
                }
                println!();
            }

            println!(
                "Use `lintbook lints --language <language>` to see rules for a specific language"
            );
            println!("Use `lintbook lints --output json` for machine-readable output");
        }
    }

    Ok(())
}

fn dump_ast(lang: Option<String>, file: Option<String>) -> anyhow::Result<()> {
    let language_name = match (&lang, &file) {
        (Some(lang), _) => lang.clone(),
        (None, Some(file_path)) => {
            let path = Path::new(file_path);
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cannot infer language: file '{}' has no extension",
                        file_path
                    )
                })?;
            lintbook_lang::get_grammar_for_extension(extension)
                .ok_or_else(|| anyhow::anyhow!("Unsupported file extension: .{}", extension))?
                .name()
                .to_string()
        }
        (None, None) => {
            return Err(anyhow::anyhow!(
                "Language must be specified when reading from stdin. Use --lang option."
            ));
        }
    };

    let grammar = lintbook_lang::get_supported_grammars()
        .into_iter()
        .find(|g| g.name() == language_name.to_lowercase())
        .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language_name))?;

    let source = if let Some(file_path) = file {
        fs::read_to_string(&file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?
    } else {
        use std::io::Read;
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {}", e))?;
        buffer
    };

    println!("{}", lintbook_lang::dump_ast(&source, grammar)?);
    Ok(())
}

fn write_if_missing(path: &Path, contents: &str) -> anyhow::Result<()> {
    if !path.exists() {
        fs::write(path, contents)?;
    }
    Ok(())
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
        if slug.len() >= 48 {
            break;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "rule".to_string()
    } else {
        slug
    }
}

async fn run_llm_lints(
    _config: &LintbookConfig,
    _repo_root: &PathBuf,
    _results: &[LintResult<Grammar>],
) -> anyhow::Result<Vec<lintbook_core::LintViolation>> {
    Ok(Vec::new())
}

const PRECOMMIT_SNIPPET: &str = r#"#!/bin/sh
lintbook check
"#;

const TEMPLATE_MD: &str = r#"---
id: rust.example
lang: rust
---

Describe the check in detail. Keep this file as prose for humans and agents.

Generate `.lintbook/gen/template.df`, then run `lintbook compile`.
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn codex_exec_args_put_approval_policy_before_exec() {
        let args = build_codex_exec_args(Path::new("/repo"), &[]);

        assert_eq!(args[0], "--ask-for-approval");
        assert_eq!(args[1], "never");
        assert_eq!(args[2], "exec");
        assert_eq!(args[3], "--cd");
        assert_eq!(args[4], "/repo");
    }

    #[test]
    fn codex_exec_args_forward_agent_args_before_exec() {
        let forwarded = vec!["--ask-for-approval".to_string(), "on-request".to_string()];
        let args = build_codex_exec_args(Path::new("/repo"), &forwarded);

        assert_eq!(args[0], "--ask-for-approval");
        assert_eq!(args[1], "on-request");
        assert_eq!(args[2], "exec");
    }

    #[test]
    fn compile_accepts_forwarded_agent_args_after_double_dash() {
        let cli = Cli::from_iter_safe([
            "lintbook",
            "compile",
            "--agent",
            "codex",
            "--",
            "--ask-for-approval",
            "on-request",
        ])
        .unwrap();

        let Cli::Compile { agent, agent_args } = cli else {
            panic!("expected compile command");
        };
        assert_eq!(agent.as_deref(), Some("codex"));
        assert_eq!(agent_args, vec!["--ask-for-approval", "on-request"]);
    }

    #[test]
    fn generated_rule_evaluation_profile_parallelizes_single_file_targets() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("main.rs");
        fs::write(&source, "fn main() {}\n").unwrap();

        assert_eq!(
            generated_rule_evaluation_profile(temp.path(), &["main.rs".to_string()]),
            lintbook_rules::GeneratedRuleEvaluationProfile::parallel()
        );
        assert_eq!(
            generated_rule_evaluation_profile(temp.path(), &[source.to_string_lossy().to_string()]),
            lintbook_rules::GeneratedRuleEvaluationProfile::parallel()
        );
    }

    #[test]
    fn generated_rule_evaluation_profile_keeps_many_file_scans_serial() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("main.rs");
        fs::write(&source, "fn main() {}\n").unwrap();

        assert_eq!(
            generated_rule_evaluation_profile(temp.path(), &[]),
            lintbook_rules::GeneratedRuleEvaluationProfile::serial()
        );
        assert_eq!(
            generated_rule_evaluation_profile(temp.path(), &[".".to_string()]),
            lintbook_rules::GeneratedRuleEvaluationProfile::serial()
        );
        assert_eq!(
            generated_rule_evaluation_profile(temp.path(), &["*.rs".to_string()]),
            lintbook_rules::GeneratedRuleEvaluationProfile::serial()
        );
        assert_eq!(
            generated_rule_evaluation_profile(
                temp.path(),
                &["main.rs".to_string(), "lib.rs".to_string()]
            ),
            lintbook_rules::GeneratedRuleEvaluationProfile::serial()
        );
    }

    #[test]
    fn codex_compile_prompt_includes_authoring_and_testing_context() {
        let temp = tempdir().unwrap();
        let rules_dir = temp.path().join(".lintbook").join("rules");
        let gen_dir = temp.path().join(".lintbook").join("gen");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::create_dir_all(&gen_dir).unwrap();
        let markdown_path = rules_dir.join("no-dbg.md");
        fs::write(
            &markdown_path,
            r#"---
id: rust.no-dbg
lang: rust
---

We don't want dbg! macro calls in production code.
"#,
        )
        .unwrap();

        let prompt = build_codex_compile_prompt(
            temp.path(),
            &[lintbook_rules::IncompleteRule {
                id: "rust.no-dbg".to_string(),
                markdown_path,
                query_path: gen_dir.join("no-dbg.df"),
            }],
        )
        .unwrap();

        assert!(prompt.contains("Datafox query grammar"));
        assert!(prompt.contains("query ::= clause"));
        assert!(prompt.contains("Available Rust facts"));
        assert!(prompt.contains("node(Node, Kind, StartLine, StartColumn, EndLine, EndColumn)"));
        assert!(prompt.contains("Example rules"));
        assert!(prompt.contains("Testing workflow"));
        assert!(prompt.contains("dump-ast --lang rust"));
        assert!(prompt.contains("check --output json <positive.rs>"));
        assert!(prompt.contains(".lintbook/gen/no-dbg.df"));
        assert!(prompt.contains("We don't want dbg! macro calls in production code."));
    }
}
