use structopt::StructOpt;
use std::env;
use treelint_scanner::{Scanner, LintStatus};
use treelint_config::TreelintConfig;
use colored::*;
use std::str::FromStr;
use serde_json;

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
            _ => Err(format!("Invalid output format: {}. Expected 'human' or 'json'", s)),
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
    },
    #[structopt(about = "Fix repository lint violations")]
    Fix,
}

fn main() -> anyhow::Result<()> {
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
        Cli::Check { output } => {
            let scan_start = std::time::Instant::now();
            
            // Only print human-readable messages if not in JSON mode
            match &output {
                OutputFormat::Human => println!("🔍 Scanning repository for lint violations..."),
                OutputFormat::Json => {},
            }
            
            let current_dir = env::current_dir()?;
            let repo_root = Scanner::find_repo_root(&current_dir)?;
            
            // Load config
            let config_path = repo_root.join("treelint.toml");
            let config = match TreelintConfig::read_from_file(&config_path) {
                Ok(cfg) => cfg,
                Err(_) => {
                    match &output {
                        OutputFormat::Human => println!("❌ No treelint.toml found. Run 'treelint init' first."),
                        OutputFormat::Json => {
                            eprintln!("Error: No treelint.toml found. Run 'treelint init' first.");
                        },
                    }
                    return Ok(());
                }
            };
            
            // Scan and lint files
            let results = Scanner::scan_and_lint_files(&repo_root, &config)?;
            
            // Display results based on output format
            match output {
                OutputFormat::Json => {
                    let total_duration = scan_start.elapsed();
                    let total_violations: usize = results.iter()
                        .map(|r| r.violations.len())
                        .sum();
                    let files_with_violations = results.iter()
                        .filter(|r| !r.violations.is_empty())
                        .count();
                    let files_scanned = results.iter()
                        .filter(|r| matches!(r.status, LintStatus::Ok | LintStatus::Error))
                        .count();
                    let files_skipped = results.iter()
                        .filter(|r| matches!(r.status, LintStatus::Skipped))
                        .count();
                    
                    let output = serde_json::json!({
                        "files": results,
                        "statistics": {
                            "total_duration_ms": total_duration.as_micros() as f64 / 1000.0,
                            "total_files": results.len(),
                            "files_scanned": files_scanned,
                            "files_skipped": files_skipped,
                            "files_with_violations": files_with_violations,
                            "total_violations": total_violations,
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
                        let file_path = result.file_path.strip_prefix(&repo_root)
                            .unwrap_or(&result.file_path)
                            .display();
                        
                        let duration_ms = result.duration.as_micros() as f64 / 1000.0;
                        
                        match result.status {
                            LintStatus::Ok => {
                                println!("{} {} {:.2}ms", file_path, "OK".green(), duration_ms);
                            }
                            LintStatus::Skipped => {
                                println!("{} {} {:.2}ms", file_path, "SKIP".dimmed(), duration_ms);
                            }
                            LintStatus::Error => {
                                println!("{} {} {:.2}ms", file_path, "ERR".red(), duration_ms);
                                for violation in &result.violations {
                                    println!("  {}:{}:{} {} ({}) - {}", 
                                        file_path.to_string().dimmed(),
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
    }
    
    Ok(())
}