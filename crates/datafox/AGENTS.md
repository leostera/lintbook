# Datafox Guide

## Scope

These instructions apply to `crates/datafox/`.

## Purpose

- `datafox` is a standalone Datalog parser and streaming query engine crate.
- Keep it independent from `poneglyph` storage details. Integration should happen through traits and adapters, not direct backend coupling.

## Working rules

- Prefer typed errors and contextual diagnostics from the start.
- Keep parser, AST, storage, and evaluator responsibilities separate.
- Treat query execution as snapshot-oriented and storage-driven, not world-materialization-driven.
