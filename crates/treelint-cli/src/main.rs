use colored::*;
use serde_json;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use treelint_config::TreelintConfig;
use treelint_scanner::Scanner;
use treelint_core::{LintResult, LintStatus};
use treelint_lang::Grammar;

#[derive(Debug, Clone)]
enum OutputFormat {
    Human,
    Json,
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
#[structopt(name = "treelint")]
#[structopt(about = "A tree-sitter based configurable linter")]
enum Cli {
    #[structopt(about = "Initialize treelint configuration for repository")]
    Init,
    #[structopt(about = "Check repository for lint violations")]
    Check {
        #[structopt(long, default_value = "human", help = "Output format (human, json)")]
        output: OutputFormat,
        #[structopt(help = "Optional file paths to check (if not provided, scans entire repository)")]
        files: Vec<String>,
    },
    #[structopt(about = "Fix repository lint violations")]
    Fix,
    #[structopt(about = "List all available lint rules")]
    Lints {
        #[structopt(long, help = "Show only rules for specific language")]
        language: Option<String>,
        #[structopt(long, default_value = "human", help = "Output format (human, json)")]
        output: OutputFormat,
    },
    #[structopt(about = "Dump AST of a file or stdin input")]
    DumpAst {
        #[structopt(long, short, help = "Language to parse (e.g., python, rust, sql). Optional when file provided - inferred from extension")]
        lang: Option<String>,
        #[structopt(help = "File to parse (if not provided, reads from stdin)")]
        file: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::from_args();

    match cli {
        Cli::Init => {
            println!("🔍 Scanning repository for supported file types...");

            let current_dir = env::current_dir()?;
            let repo_root = Scanner::find_repo_root(&current_dir)?;
            println!("📁 Found repository root: {}", repo_root.display());

            let grammars = Scanner::detect_grammars(&repo_root)?;

            if grammars.is_empty() {
                println!("❌ No supported file types found in repository");
                return Ok(());
            }

            println!("✅ Detected {} grammar(s):", grammars.len());
            for grammar in &grammars {
                println!("  • {}", grammar.name());
            }

            let grammar_names: Vec<&str> = grammars.iter().map(|g| g.name()).collect();
            let config = TreelintConfig::new(grammar_names);
            let config_path = repo_root.join("treelint.toml");
            config.write_to_file(&config_path)?;

            println!("📝 Created configuration file: {}", config_path.display());
            println!("🎉 treelint initialization complete!");
        }
        Cli::Check { output, files } => {
            let scan_start = std::time::Instant::now();

            // Only print human-readable messages if not in JSON mode
            match &output {
                OutputFormat::Human => println!("🔍 Scanning repository for lint violations..."),
                OutputFormat::Json => {}
            }

            let current_dir = env::current_dir()?;
            let repo_root = Scanner::find_repo_root(&current_dir)?;

            // Load config
            let config_path = repo_root.join("treelint.toml");
            let config = match TreelintConfig::read_from_file(&config_path) {
                Ok(cfg) => cfg,
                Err(_) => {
                    match &output {
                        OutputFormat::Human => {
                            println!("❌ No treelint.toml found. Run 'treelint init' first.")
                        }
                        OutputFormat::Json => {
                            eprintln!("Error: No treelint.toml found. Run 'treelint init' first.");
                        }
                    }
                    return Ok(());
                }
            };

            // Scan and lint files
            let results = if files.is_empty() {
                // Use scanner to find files
                Scanner::scan_and_lint_files(&repo_root, &config)?
            } else {
                // Process specific files directly without scanner
                let mut results = Vec::new();
                
                for file_path_str in &files {
                    let file_path = if std::path::Path::new(file_path_str).is_absolute() {
                        std::path::PathBuf::from(file_path_str)
                    } else {
                        repo_root.join(file_path_str)
                    };

                    // Skip if file doesn't exist
                    if !file_path.exists() {
                        eprintln!("Warning: File not found: {}", file_path.display());
                        continue;
                    }

                    // Skip directories
                    if file_path.is_dir() {
                        eprintln!("Warning: Skipping directory: {}", file_path.display());
                        continue;
                    }

                    // Check file extension
                    if let Some(extension) = file_path.extension() {
                        let ext = extension.to_string_lossy();

                        // Check if we support this language
                        if let Some(grammar) = treelint_lang::get_grammar_for_extension(&ext) {
                            // Check if this language is enabled in config
                            if config.treelint.languages.contains(&grammar.name().to_string()) {
                                let start_time = std::time::Instant::now();

                                // Read file and parse/lint using treelint_lang
                                match std::fs::read_to_string(&file_path) {
                                    Ok(source) => {
                                        results.push(treelint_lang::parse(&config, &file_path, &source, grammar, start_time));
                                    }
                                    Err(e) => {
                                        eprintln!("Error reading file {}: {}", file_path.display(), e);
                                    }
                                }
                            }
                        }
                    }
                }
                
                results
            };

            // Run LLM lints if configured
            let llm_violations = run_llm_lints(&config, &repo_root, &results).await?;

            // Display results based on output format
            match output {
                OutputFormat::Json => {
                    let total_duration = scan_start.elapsed();
                    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum::<usize>() + llm_violations.len();
                    let files_with_violations =
                        results.iter().filter(|r| !r.violations.is_empty()).count();
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

                    let json_output = serde_json::to_string_pretty(&output)?;
                    println!("{}", json_output);

                    if total_violations > 0 {
                        std::process::exit(1);
                    }
                }
                OutputFormat::Human => {
                    let mut total_violations = 0;

                    for result in &results {
                        let file_path = result
                            .file_path
                            .strip_prefix(&repo_root)
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
                                    // Extract just the filename from the path
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

                    // Display LLM violations (currently disabled)
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

                    let total_duration = scan_start.elapsed();
                    let total_ms = total_duration.as_micros() as f64 / 1000.0;
                    println!("\n✨ Treelinted project in {:.2}ms", total_ms);

                    if total_violations > 0 {
                        std::process::exit(1);
                    }
                }
            }
        }
        Cli::Fix => {
            println!("Fixing repository lint violations...");
        }
        Cli::Lints { language, output } => {
            let mut all_lints = Vec::new();

            // Get lints for specific language or all languages
            if let Some(lang) = language {
                // Try to find the grammar for the specified language
                let grammar = treelint_lang::get_supported_grammars()
                    .into_iter()
                    .find(|g| g.name() == lang.to_lowercase());

                if let Some(grammar) = grammar {
                    let lints = grammar.lints();
                    all_lints.extend(lints.into_iter().map(|lint| (grammar.name(), lint)));
                } else {
                    match output {
                        OutputFormat::Human => {
                            println!("❌ Unknown language: {}. Use 'treelint lints' to see all supported languages.", lang);
                        }
                        OutputFormat::Json => {
                            eprintln!("Error: Unknown language: {}", lang);
                        }
                    }
                    return Ok(());
                }
            } else {
                // Get lints for all supported grammars
                for grammar in treelint_lang::get_supported_grammars() {
                    let lints = grammar.lints();
                    all_lints.extend(lints.into_iter().map(|lint| (grammar.name(), lint)));
                }
            }

            // Display results based on output format
            match output {
                OutputFormat::Json => {
                    let mut languages = std::collections::HashMap::new();

                    for (lang, lint) in all_lints {
                        let lang_entry = languages.entry(lang).or_insert_with(Vec::new);
                        lang_entry.push(serde_json::json!({
                            "id": lint.id(),
                            "name": lint.name(),
                            "description": lint.description(),
                            "explanation": lint.explanation()
                        }));
                    }

                    let output = serde_json::json!({
                        "languages": languages
                    });

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                OutputFormat::Human => {
                    if all_lints.is_empty() {
                        println!("No lint rules available.");
                        return Ok(());
                    }

                    println!("📋 Available Lint Rules\n");

                    // Group lints by language to count them
                    let mut language_lints: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
                    for (lang, lint) in &all_lints {
                        language_lints.entry(lang).or_insert_with(Vec::new).push(lint);
                    }

                    // Sort languages for consistent output
                    let mut languages: Vec<_> = language_lints.keys().cloned().collect();
                    languages.sort();

                    for lang in languages {
                        let lints = &language_lints[lang];
                        println!("🔍 {} Rules ({}):", lang.to_uppercase(), lints.len());
                        
                        for lint in lints {
                            println!(
                                "  {} {} - {}",
                                lint.id().bright_cyan(),
                                lint.name().cyan(),
                                lint.description()
                            );
                        }
                        println!();
                    }

                    println!("💡 Use 'treelint lints --language <language>' to see rules for a specific language");
                    println!("💡 Use 'treelint lints --output json' for machine-readable output");
                }
            }
        }
        Cli::DumpAst { lang, file } => {
            // Determine the language to use
            let language_name = match (&lang, &file) {
                // Language explicitly provided
                (Some(lang), _) => lang.clone(),
                // No language provided, but we have a file - infer from extension
                (None, Some(file_path)) => {
                    let path = std::path::Path::new(file_path);
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .ok_or_else(|| anyhow::anyhow!("Cannot infer language: file '{}' has no extension", file_path))?;
                    
                    // Find grammar by extension
                    let grammar = treelint_lang::get_grammar_for_extension(extension)
                        .ok_or_else(|| anyhow::anyhow!("Unsupported file extension: .{}", extension))?;
                    
                    grammar.name().to_string()
                },
                // No language and no file - cannot proceed
                (None, None) => {
                    return Err(anyhow::anyhow!("Language must be specified when reading from stdin. Use --lang option."));
                }
            };

            // Get the grammar for the determined language
            let grammar = treelint_lang::get_supported_grammars()
                .into_iter()
                .find(|g| g.name() == language_name.to_lowercase())
                .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language_name))?;

            // Read input from file or stdin
            let source = if let Some(file_path) = file {
                std::fs::read_to_string(&file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?
            } else {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)
                    .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {}", e))?;
                buffer
            };

            // Dump AST as JSON
            let ast_json = treelint_lang::dump_ast(&source, grammar)?;
            println!("{}", ast_json);
        }
    }

    Ok(())
}

async fn run_llm_lints(
    _config: &TreelintConfig,
    _repo_root: &PathBuf,
    _results: &[LintResult<Grammar>],
) -> anyhow::Result<Vec<treelint_core::LintViolation>> {
    // LLM functionality temporarily disabled until implementation is complete
    Ok(Vec::new())
}
