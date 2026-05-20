use lintbook_config::LintbookConfig;
use lintbook_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Grammar {
    Bash,
    C,
    CSharp,
    Cpp,
    Css,
    Go,
    Html,
    Java,
    Javascript,
    Json,
    Markdown,
    Python,
    Ruby,
    Rust,
    Toml,
    Typescript,
    Yaml,
    Dockerfile,
    Lua,
    Nix,
    Vim,
    Xml,
    Fish,
    Scala,
    Elixir,
    Erlang,
    Ocaml,
    Haskell,
    Elm,
    Gleam,
    Julia,
    Php,
    Clojure,
    Dart,
    Zig,
    Cmake,
    Scss,
    Sql,
    Svelte,
    Vue,
}

impl Grammar {
    pub fn name(&self) -> &'static str {
        match self {
            Grammar::Bash => "bash",
            Grammar::C => "c",
            Grammar::CSharp => "c-sharp",
            Grammar::Cpp => "cpp",
            Grammar::Css => "css",
            Grammar::Go => "go",
            Grammar::Html => "html",
            Grammar::Java => "java",
            Grammar::Javascript => "javascript",
            Grammar::Json => "json",
            Grammar::Markdown => "markdown",
            Grammar::Python => "python",
            Grammar::Ruby => "ruby",
            Grammar::Rust => "rust",
            Grammar::Toml => "toml",
            Grammar::Typescript => "typescript",
            Grammar::Yaml => "yaml",
            Grammar::Dockerfile => "dockerfile",
            Grammar::Lua => "lua",
            Grammar::Nix => "nix",
            Grammar::Vim => "vim",
            Grammar::Xml => "xml",
            Grammar::Fish => "fish",
            Grammar::Scala => "scala",
            Grammar::Elixir => "elixir",
            Grammar::Erlang => "erlang",
            Grammar::Ocaml => "ocaml",
            Grammar::Haskell => "haskell",
            Grammar::Elm => "elm",
            Grammar::Gleam => "gleam",
            Grammar::Julia => "julia",
            Grammar::Php => "php",
            Grammar::Clojure => "clojure",
            Grammar::Dart => "dart",
            Grammar::Zig => "zig",
            Grammar::Cmake => "cmake",
            Grammar::Scss => "scss",
            Grammar::Sql => "sql",
            Grammar::Svelte => "svelte",
            Grammar::Vue => "vue",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Grammar::Bash => &["sh", "bash", "zsh"],
            Grammar::C => &["c", "h"],
            Grammar::CSharp => &["cs"],
            Grammar::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx"],
            Grammar::Css => &["css"],
            Grammar::Go => &["go"],
            Grammar::Html => &["html", "htm"],
            Grammar::Java => &["java"],
            Grammar::Javascript => &["js", "mjs"],
            Grammar::Json => &["json"],
            Grammar::Markdown => &["md", "markdown"],
            Grammar::Python => &["py", "pyi"],
            Grammar::Ruby => &["rb"],
            Grammar::Rust => &["rs"],
            Grammar::Toml => &["toml"],
            Grammar::Typescript => &["ts", "tsx"],
            Grammar::Yaml => &["yaml", "yml"],
            Grammar::Dockerfile => &["dockerfile"],
            Grammar::Lua => &["lua"],
            Grammar::Nix => &["nix"],
            Grammar::Vim => &["vim"],
            Grammar::Xml => &["xml"],
            Grammar::Fish => &["fish"],
            Grammar::Scala => &["scala", "sc"],
            Grammar::Elixir => &["ex", "exs"],
            Grammar::Erlang => &["erl"],
            Grammar::Ocaml => &["ml", "mli"],
            Grammar::Haskell => &["hs"],
            Grammar::Elm => &["elm"],
            Grammar::Gleam => &["gleam"],
            Grammar::Julia => &["jl"],
            Grammar::Php => &["php"],
            Grammar::Clojure => &["clj", "cljs"],
            Grammar::Dart => &["dart"],
            Grammar::Zig => &["zig"],
            Grammar::Cmake => &["cmake"],
            Grammar::Scss => &["scss"],
            Grammar::Sql => &["sql"],
            Grammar::Svelte => &["svelte"],
            Grammar::Vue => &["vue"],
        }
    }

    pub fn lints(&self) -> Vec<Box<dyn Rule>> {
        Vec::new()
    }

    pub fn from_name(name: &str) -> anyhow::Result<Self> {
        get_supported_grammars()
            .into_iter()
            .find(|grammar| grammar.name() == name)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", name))
    }

    pub fn to_tree_sitter_language(&self) -> anyhow::Result<tree_sitter::Language> {
        match self {
            Grammar::Python => Ok(tree_sitter_python::LANGUAGE.into()),
            Grammar::Elixir => Ok(tree_sitter_elixir::LANGUAGE.into()),
            Grammar::Sql => Ok(tree_sitter_sequel::LANGUAGE.into()),
            Grammar::Rust => Ok(tree_sitter_rust::LANGUAGE.into()),
            _ => Err(anyhow::anyhow!(
                "No parser is wired for language: {}",
                self.name()
            )),
        }
    }
}

pub fn get_supported_grammars() -> Vec<Grammar> {
    vec![
        Grammar::Bash,
        Grammar::C,
        Grammar::CSharp,
        Grammar::Cpp,
        Grammar::Css,
        Grammar::Go,
        Grammar::Html,
        Grammar::Java,
        Grammar::Javascript,
        Grammar::Json,
        Grammar::Markdown,
        Grammar::Python,
        Grammar::Ruby,
        Grammar::Rust,
        Grammar::Toml,
        Grammar::Typescript,
        Grammar::Yaml,
        Grammar::Dockerfile,
        Grammar::Lua,
        Grammar::Nix,
        Grammar::Vim,
        Grammar::Xml,
        Grammar::Fish,
        Grammar::Scala,
        Grammar::Elixir,
        Grammar::Erlang,
        Grammar::Ocaml,
        Grammar::Haskell,
        Grammar::Elm,
        Grammar::Gleam,
        Grammar::Julia,
        Grammar::Php,
        Grammar::Clojure,
        Grammar::Dart,
        Grammar::Zig,
        Grammar::Cmake,
        Grammar::Scss,
        Grammar::Sql,
        Grammar::Svelte,
        Grammar::Vue,
    ]
}

pub fn get_grammar_for_extension(extension: &str) -> Option<Grammar> {
    let grammars = get_supported_grammars();
    let extension_map: HashMap<&str, Grammar> = grammars
        .iter()
        .flat_map(|grammar| grammar.extensions().iter().map(move |ext| (*ext, *grammar)))
        .collect();

    extension_map.get(extension).copied()
}

pub fn get_grammars_for_extensions(extensions: &[String]) -> Vec<Grammar> {
    extensions
        .iter()
        .filter_map(|ext| get_grammar_for_extension(ext))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

pub fn parse(
    config: &LintbookConfig,
    path: &Path,
    source: &str,
    grammar: Grammar,
    start_time: Instant,
) -> LintResult<Grammar> {
    let language = match grammar.to_tree_sitter_language() {
        Ok(language) => language,
        Err(_) => {
            return LintResult {
                file_path: path.to_path_buf(),
                duration: start_time.elapsed(),
                status: LintStatus::Skipped,
                violations: vec![],
                language: None,
            };
        }
    };

    let lints = grammar.lints();
    if lints.is_empty() {
        return LintResult {
            file_path: path.to_path_buf(),
            duration: start_time.elapsed(),
            status: LintStatus::Ok,
            violations: vec![],
            language: Some(grammar),
        };
    }

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();

    let tree = parser.parse(&source, None).unwrap();
    let mut all_violations = Vec::new();

    for lint in lints {
        if config.is_lint_enabled(grammar.name(), lint.name()) {
            let mut violations = lint.check(&tree, &source);
            // Ensure violations have the correct lint metadata
            for v in &mut violations {
                v.lint_id = lint.id().to_string();
                v.lint_name = lint.name().to_string();
            }
            all_violations.extend(violations);
        }
    }

    let status = if all_violations.is_empty() {
        LintStatus::Ok
    } else {
        LintStatus::Error
    };

    LintResult {
        file_path: path.to_path_buf(),
        duration: start_time.elapsed(),
        status,
        violations: all_violations,
        language: Some(grammar),
    }
}

pub fn dump_ast(source: &str, grammar: Grammar) -> anyhow::Result<String> {
    let language = grammar.to_tree_sitter_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language)?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse source"))?;

    let root = dump_node(tree.root_node(), source);
    Ok(serde_json::to_string_pretty(&root)?)
}

fn dump_node(node: tree_sitter::Node<'_>, source: &str) -> serde_json::Value {
    let mut children = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        children.push(dump_node(child, source));
    }

    let start = node.start_position();
    let end = node.end_position();
    let text = if children.is_empty() {
        Some(&source[node.byte_range()])
    } else {
        None
    };

    serde_json::json!({
        "kind": node.kind(),
        "is_named": node.is_named(),
        "start": {
            "row": start.row,
            "column": start.column,
        },
        "end": {
            "row": end.row,
            "column": end.column,
        },
        "start_byte": node.start_byte(),
        "end_byte": node.end_byte(),
        "text": text,
        "children": children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_name() {
        assert_eq!(Grammar::Python.name(), "python");
        assert_eq!(Grammar::Rust.name(), "rust");
        assert_eq!(Grammar::Javascript.name(), "javascript");
    }

    #[test]
    fn test_grammar_extensions() {
        assert_eq!(Grammar::Python.extensions(), &["py", "pyi"]);
        assert_eq!(Grammar::Rust.extensions(), &["rs"]);
        assert_eq!(Grammar::Javascript.extensions(), &["js", "mjs"]);
    }

    #[test]
    fn test_get_grammar_for_extension() {
        assert_eq!(get_grammar_for_extension("py"), Some(Grammar::Python));
        assert_eq!(get_grammar_for_extension("rs"), Some(Grammar::Rust));
        assert_eq!(get_grammar_for_extension("js"), Some(Grammar::Javascript));
        assert_eq!(get_grammar_for_extension("unknown"), None);
    }

    #[test]
    fn test_get_grammars_for_extensions() {
        let extensions = vec!["py".to_string(), "rs".to_string(), "unknown".to_string()];
        let grammars = get_grammars_for_extensions(&extensions);

        assert!(grammars.contains(&Grammar::Python));
        assert!(grammars.contains(&Grammar::Rust));
        assert_eq!(grammars.len(), 2);
    }

    #[test]
    fn test_language_specific_rust_lints_are_disabled() {
        assert!(Grammar::Python.lints().is_empty());
        assert!(Grammar::Rust.lints().is_empty());
        assert!(Grammar::Elixir.lints().is_empty());
        assert!(Grammar::Sql.lints().is_empty());
    }

    #[test]
    fn test_unsupported_grammar_has_no_lints() {
        let lints = Grammar::Json.lints();
        assert!(lints.is_empty());
    }
}
