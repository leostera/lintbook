# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# treelint

## Project Overview

This project is a tree-sitter based configurable linter for any programming language, designed to be fast enough for precommit hooks and CI pipelines.

## Architecture

- **Language**: Rust (for performance)
- **CLI tool**: Single binary called `treelint`
- **Configuration**: `treelint.toml` file
- **Distribution**: GitHub workflows for releasing statically linked binaries for all major architectures

## Key Commands

The primary CLI commands planned:
- `treelint init` - Scans repository, detects languages, and creates initial `treelint.toml` configuration
- `treelint` - Runs linting on configured files

## Current State

This is an early-stage project with only basic project files (README, LICENSE, CLAUDE.md). The core Rust implementation, configuration system, and CI/CD workflows are yet to be implemented.

## Development Notes

- The project will ship with built-in lints for common languages
- Plugin system is planned but implementation approach is undecided (query language vs JavaScript)
- Focus on performance to enable use in precommit hooks and CI pipelines

## Git Workflow

- Commit often and with good messages