pub mod lints;

use lints::Rule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            Grammar::Svelte => &["svelte"],
            Grammar::Vue => &["vue"],
        }
    }

    pub fn get_lints(&self) -> Vec<Box<dyn Rule>> {
        match self {
            Grammar::Python => lints::python::get_python_lints(),
            _ => vec![],
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
    fn test_python_has_lints() {
        let lints = Grammar::Python.get_lints();
        assert!(!lints.is_empty());
    }

    #[test]
    fn test_unsupported_grammar_has_no_lints() {
        let lints = Grammar::Json.get_lints();
        assert!(lints.is_empty());
    }
}
