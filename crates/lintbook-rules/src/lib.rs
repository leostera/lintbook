use anyhow::{anyhow, bail, Context, Result};
use datafox::{
    Clause, DatafoxClient, DatafoxConfig, DatafoxEnvironment, InMemoryPreparedQueryStorage,
    InMemoryStorage, PreparedQuery, PreparedQueryKey, PreparedQueryStorage, Query, Substitution,
    Term, Value,
};
use lintbook_config::LintbookConfig;
use lintbook_core::{LintResult, LintStatus, LintViolation};
use lintbook_lang::Grammar;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tree_sitter::{Node, Parser};

const SCHEMA_VERSION: u32 = 2;
const FACT_SCHEMA_VERSION: u32 = 10;
const BINCODE_CACHE_FORMAT_VERSION: u32 = 2;
const LINTBOOK_DIR: &str = ".lintbook";
const RULES_DIR: &str = "rules";
const GEN_DIR: &str = "gen";
const CACHE_DIR: &str = "cache";
const FACTS_DIR: &str = "facts";
const PREPARED_QUERIES_DIR: &str = "prepared-queries";

const BUILTIN_RULES: &[BuiltinRuleAsset] = &[
    BuiltinRuleAsset {
        name: "no-try-catch",
        markdown_path: "builtin/python/py001-no-try-catch.md",
        query_path: "builtin/python/py001-no-try-catch.df",
        markdown: include_str!("../builtin/python/py001-no-try-catch.md"),
        query: include_str!("../builtin/python/py001-no-try-catch.df"),
    },
    BuiltinRuleAsset {
        name: "no-sys-path-modification",
        markdown_path: "builtin/python/py002-no-sys-path-modification.md",
        query_path: "builtin/python/py002-no-sys-path-modification.df",
        markdown: include_str!("../builtin/python/py002-no-sys-path-modification.md"),
        query: include_str!("../builtin/python/py002-no-sys-path-modification.df"),
    },
    BuiltinRuleAsset {
        name: "absurd-extreme-comparisons",
        markdown_path: "builtin/rust/rs001-absurd-extreme-comparisons.md",
        query_path: "builtin/rust/rs001-absurd-extreme-comparisons.df",
        markdown: include_str!("../builtin/rust/rs001-absurd-extreme-comparisons.md"),
        query: include_str!("../builtin/rust/rs001-absurd-extreme-comparisons.df"),
    },
    BuiltinRuleAsset {
        name: "almost-swapped",
        markdown_path: "builtin/rust/rs002-almost-swapped.md",
        query_path: "builtin/rust/rs002-almost-swapped.df",
        markdown: include_str!("../builtin/rust/rs002-almost-swapped.md"),
        query: include_str!("../builtin/rust/rs002-almost-swapped.df"),
    },
    BuiltinRuleAsset {
        name: "approx-constant",
        markdown_path: "builtin/rust/rs003-approx-constant.md",
        query_path: "builtin/rust/rs003-approx-constant.df",
        markdown: include_str!("../builtin/rust/rs003-approx-constant.md"),
        query: include_str!("../builtin/rust/rs003-approx-constant.df"),
    },
    BuiltinRuleAsset {
        name: "async-yields-async",
        markdown_path: "builtin/rust/rs004-async-yields-async.md",
        query_path: "builtin/rust/rs004-async-yields-async.df",
        markdown: include_str!("../builtin/rust/rs004-async-yields-async.md"),
        query: include_str!("../builtin/rust/rs004-async-yields-async.df"),
    },
    BuiltinRuleAsset {
        name: "eq-op",
        markdown_path: "builtin/rust/rs013-eq-op.md",
        query_path: "builtin/rust/rs013-eq-op.df",
        markdown: include_str!("../builtin/rust/rs013-eq-op.md"),
        query: include_str!("../builtin/rust/rs013-eq-op.df"),
    },
    BuiltinRuleAsset {
        name: "erasing-op",
        markdown_path: "builtin/rust/rs014-erasing-op.md",
        query_path: "builtin/rust/rs014-erasing-op.df",
        markdown: include_str!("../builtin/rust/rs014-erasing-op.md"),
        query: include_str!("../builtin/rust/rs014-erasing-op.df"),
    },
    BuiltinRuleAsset {
        name: "ifs-same-cond",
        markdown_path: "builtin/rust/rs016-ifs-same-cond.md",
        query_path: "builtin/rust/rs016-ifs-same-cond.df",
        markdown: include_str!("../builtin/rust/rs016-ifs-same-cond.md"),
        query: include_str!("../builtin/rust/rs016-ifs-same-cond.df"),
    },
    BuiltinRuleAsset {
        name: "inline-fn-without-body",
        markdown_path: "builtin/rust/rs022-inline-fn-without-body.md",
        query_path: "builtin/rust/rs022-inline-fn-without-body.df",
        markdown: include_str!("../builtin/rust/rs022-inline-fn-without-body.md"),
        query: include_str!("../builtin/rust/rs022-inline-fn-without-body.df"),
    },
    BuiltinRuleAsset {
        name: "invisible-characters",
        markdown_path: "builtin/rust/rs025-invisible-characters.md",
        query_path: "builtin/rust/rs025-invisible-characters.df",
        markdown: include_str!("../builtin/rust/rs025-invisible-characters.md"),
        query: include_str!("../builtin/rust/rs025-invisible-characters.df"),
    },
    BuiltinRuleAsset {
        name: "iter-next-loop",
        markdown_path: "builtin/rust/rs026-iter-next-loop.md",
        query_path: "builtin/rust/rs026-iter-next-loop.df",
        markdown: include_str!("../builtin/rust/rs026-iter-next-loop.md"),
        query: include_str!("../builtin/rust/rs026-iter-next-loop.df"),
    },
    BuiltinRuleAsset {
        name: "iter-skip-zero",
        markdown_path: "builtin/rust/rs027-iter-skip-zero.md",
        query_path: "builtin/rust/rs027-iter-skip-zero.df",
        markdown: include_str!("../builtin/rust/rs027-iter-skip-zero.md"),
        query: include_str!("../builtin/rust/rs027-iter-skip-zero.df"),
    },
    BuiltinRuleAsset {
        name: "iterator-step-by-zero",
        markdown_path: "builtin/rust/rs028-iterator-step-by-zero.md",
        query_path: "builtin/rust/rs028-iterator-step-by-zero.df",
        markdown: include_str!("../builtin/rust/rs028-iterator-step-by-zero.md"),
        query: include_str!("../builtin/rust/rs028-iterator-step-by-zero.df"),
    },
    BuiltinRuleAsset {
        name: "let-underscore-lock",
        markdown_path: "builtin/rust/rs029-let-underscore-lock.md",
        query_path: "builtin/rust/rs029-let-underscore-lock.df",
        markdown: include_str!("../builtin/rust/rs029-let-underscore-lock.md"),
        query: include_str!("../builtin/rust/rs029-let-underscore-lock.df"),
    },
    BuiltinRuleAsset {
        name: "mem-replace-with-uninit",
        markdown_path: "builtin/rust/rs032-mem-replace-with-uninit.md",
        query_path: "builtin/rust/rs032-mem-replace-with-uninit.df",
        markdown: include_str!("../builtin/rust/rs032-mem-replace-with-uninit.md"),
        query: include_str!("../builtin/rust/rs032-mem-replace-with-uninit.df"),
    },
    BuiltinRuleAsset {
        name: "mistyped-literal-suffixes",
        markdown_path: "builtin/rust/rs034-mistyped-literal-suffixes.md",
        query_path: "builtin/rust/rs034-mistyped-literal-suffixes.df",
        markdown: include_str!("../builtin/rust/rs034-mistyped-literal-suffixes.md"),
        query: include_str!("../builtin/rust/rs034-mistyped-literal-suffixes.df"),
    },
    BuiltinRuleAsset {
        name: "modulo-one",
        markdown_path: "builtin/rust/rs035-modulo-one.md",
        query_path: "builtin/rust/rs035-modulo-one.df",
        markdown: include_str!("../builtin/rust/rs035-modulo-one.md"),
        query: include_str!("../builtin/rust/rs035-modulo-one.df"),
    },
    BuiltinRuleAsset {
        name: "non-octal-unix-permissions",
        markdown_path: "builtin/rust/rs038-non-octal-unix-permissions.md",
        query_path: "builtin/rust/rs038-non-octal-unix-permissions.df",
        markdown: include_str!("../builtin/rust/rs038-non-octal-unix-permissions.md"),
        query: include_str!("../builtin/rust/rs038-non-octal-unix-permissions.df"),
    },
    BuiltinRuleAsset {
        name: "option-env-unwrap",
        markdown_path: "builtin/rust/rs041-option-env-unwrap.md",
        query_path: "builtin/rust/rs041-option-env-unwrap.df",
        markdown: include_str!("../builtin/rust/rs041-option-env-unwrap.md"),
        query: include_str!("../builtin/rust/rs041-option-env-unwrap.df"),
    },
    BuiltinRuleAsset {
        name: "possible-missing-comma",
        markdown_path: "builtin/rust/rs046-possible-missing-comma.md",
        query_path: "builtin/rust/rs046-possible-missing-comma.df",
        markdown: include_str!("../builtin/rust/rs046-possible-missing-comma.md"),
        query: include_str!("../builtin/rust/rs046-possible-missing-comma.df"),
    },
    BuiltinRuleAsset {
        name: "reversed-empty-ranges",
        markdown_path: "builtin/rust/rs050-reversed-empty-ranges.md",
        query_path: "builtin/rust/rs050-reversed-empty-ranges.df",
        markdown: include_str!("../builtin/rust/rs050-reversed-empty-ranges.md"),
        query: include_str!("../builtin/rust/rs050-reversed-empty-ranges.df"),
    },
    BuiltinRuleAsset {
        name: "self-assignment",
        markdown_path: "builtin/rust/rs051-self-assignment.md",
        query_path: "builtin/rust/rs051-self-assignment.df",
        markdown: include_str!("../builtin/rust/rs051-self-assignment.md"),
        query: include_str!("../builtin/rust/rs051-self-assignment.df"),
    },
    BuiltinRuleAsset {
        name: "suspicious-splitn",
        markdown_path: "builtin/rust/rs054-suspicious-splitn.md",
        query_path: "builtin/rust/rs054-suspicious-splitn.df",
        markdown: include_str!("../builtin/rust/rs054-suspicious-splitn.md"),
        query: include_str!("../builtin/rust/rs054-suspicious-splitn.df"),
    },
    BuiltinRuleAsset {
        name: "transmute-null-to-fn",
        markdown_path: "builtin/rust/rs055-transmute-null-to-fn.md",
        query_path: "builtin/rust/rs055-transmute-null-to-fn.df",
        markdown: include_str!("../builtin/rust/rs055-transmute-null-to-fn.md"),
        query: include_str!("../builtin/rust/rs055-transmute-null-to-fn.df"),
    },
    BuiltinRuleAsset {
        name: "transmuting-null",
        markdown_path: "builtin/rust/rs056-transmuting-null.md",
        query_path: "builtin/rust/rs056-transmuting-null.df",
        markdown: include_str!("../builtin/rust/rs056-transmuting-null.md"),
        query: include_str!("../builtin/rust/rs056-transmuting-null.df"),
    },
    BuiltinRuleAsset {
        name: "uninit-assumed-init",
        markdown_path: "builtin/rust/rs057-uninit-assumed-init.md",
        query_path: "builtin/rust/rs057-uninit-assumed-init.df",
        markdown: include_str!("../builtin/rust/rs057-uninit-assumed-init.md"),
        query: include_str!("../builtin/rust/rs057-uninit-assumed-init.df"),
    },
    BuiltinRuleAsset {
        name: "uninit-vec",
        markdown_path: "builtin/rust/rs058-uninit-vec.md",
        query_path: "builtin/rust/rs058-uninit-vec.df",
        markdown: include_str!("../builtin/rust/rs058-uninit-vec.md"),
        query: include_str!("../builtin/rust/rs058-uninit-vec.df"),
    },
    BuiltinRuleAsset {
        name: "unit-cmp",
        markdown_path: "builtin/rust/rs059-unit-cmp.md",
        query_path: "builtin/rust/rs059-unit-cmp.df",
        markdown: include_str!("../builtin/rust/rs059-unit-cmp.md"),
        query: include_str!("../builtin/rust/rs059-unit-cmp.df"),
    },
    BuiltinRuleAsset {
        name: "unit-hash",
        markdown_path: "builtin/rust/rs060-unit-hash.md",
        query_path: "builtin/rust/rs060-unit-hash.df",
        markdown: include_str!("../builtin/rust/rs060-unit-hash.md"),
        query: include_str!("../builtin/rust/rs060-unit-hash.df"),
    },
    BuiltinRuleAsset {
        name: "unused-io-amount",
        markdown_path: "builtin/rust/rs063-unused-io-amount.md",
        query_path: "builtin/rust/rs063-unused-io-amount.df",
        markdown: include_str!("../builtin/rust/rs063-unused-io-amount.md"),
        query: include_str!("../builtin/rust/rs063-unused-io-amount.df"),
    },
    BuiltinRuleAsset {
        name: "useless-attribute",
        markdown_path: "builtin/rust/rs064-useless-attribute.md",
        query_path: "builtin/rust/rs064-useless-attribute.df",
        markdown: include_str!("../builtin/rust/rs064-useless-attribute.md"),
        query: include_str!("../builtin/rust/rs064-useless-attribute.df"),
    },
    BuiltinRuleAsset {
        name: "vec-resize-to-zero",
        markdown_path: "builtin/rust/rs065-vec-resize-to-zero.md",
        query_path: "builtin/rust/rs065-vec-resize-to-zero.df",
        markdown: include_str!("../builtin/rust/rs065-vec-resize-to-zero.md"),
        query: include_str!("../builtin/rust/rs065-vec-resize-to-zero.df"),
    },
    BuiltinRuleAsset {
        name: "cast-abs-to-unsigned",
        markdown_path: "builtin/rust/rs075-cast-abs-to-unsigned.md",
        query_path: "builtin/rust/rs075-cast-abs-to-unsigned.df",
        markdown: include_str!("../builtin/rust/rs075-cast-abs-to-unsigned.md"),
        query: include_str!("../builtin/rust/rs075-cast-abs-to-unsigned.df"),
    },
    BuiltinRuleAsset {
        name: "cast-slice-from-raw-parts",
        markdown_path: "builtin/rust/rs079-cast-slice-from-raw-parts.md",
        query_path: "builtin/rust/rs079-cast-slice-from-raw-parts.df",
        markdown: include_str!("../builtin/rust/rs079-cast-slice-from-raw-parts.md"),
        query: include_str!("../builtin/rust/rs079-cast-slice-from-raw-parts.df"),
    },
    BuiltinRuleAsset {
        name: "const-is-empty",
        markdown_path: "builtin/rust/rs081-const-is-empty.md",
        query_path: "builtin/rust/rs081-const-is-empty.df",
        markdown: include_str!("../builtin/rust/rs081-const-is-empty.md"),
        query: include_str!("../builtin/rust/rs081-const-is-empty.df"),
    },
    BuiltinRuleAsset {
        name: "crate-in-macro-def",
        markdown_path: "builtin/rust/rs082-crate-in-macro-def.md",
        query_path: "builtin/rust/rs082-crate-in-macro-def.df",
        markdown: include_str!("../builtin/rust/rs082-crate-in-macro-def.md"),
        query: include_str!("../builtin/rust/rs082-crate-in-macro-def.df"),
    },
    BuiltinRuleAsset {
        name: "deprecated-clippy-cfg-attr",
        markdown_path: "builtin/rust/rs083-deprecated-clippy-cfg-attr.md",
        query_path: "builtin/rust/rs083-deprecated-clippy-cfg-attr.df",
        markdown: include_str!("../builtin/rust/rs083-deprecated-clippy-cfg-attr.md"),
        query: include_str!("../builtin/rust/rs083-deprecated-clippy-cfg-attr.df"),
    },
    BuiltinRuleAsset {
        name: "duplicate-mod",
        markdown_path: "builtin/rust/rs087-duplicate-mod.md",
        query_path: "builtin/rust/rs087-duplicate-mod.df",
        markdown: include_str!("../builtin/rust/rs087-duplicate-mod.md"),
        query: include_str!("../builtin/rust/rs087-duplicate-mod.df"),
    },
    BuiltinRuleAsset {
        name: "duplicated-attributes",
        markdown_path: "builtin/rust/rs088-duplicated-attributes.md",
        query_path: "builtin/rust/rs088-duplicated-attributes.df",
        markdown: include_str!("../builtin/rust/rs088-duplicated-attributes.md"),
        query: include_str!("../builtin/rust/rs088-duplicated-attributes.df"),
    },
    BuiltinRuleAsset {
        name: "empty-docs",
        markdown_path: "builtin/rust/rs089-empty-docs.md",
        query_path: "builtin/rust/rs089-empty-docs.df",
        markdown: include_str!("../builtin/rust/rs089-empty-docs.md"),
        query: include_str!("../builtin/rust/rs089-empty-docs.df"),
    },
    BuiltinRuleAsset {
        name: "empty-line-after-doc-comments",
        markdown_path: "builtin/rust/rs090-empty-line-after-doc-comments.md",
        query_path: "builtin/rust/rs090-empty-line-after-doc-comments.df",
        markdown: include_str!("../builtin/rust/rs090-empty-line-after-doc-comments.md"),
        query: include_str!("../builtin/rust/rs090-empty-line-after-doc-comments.df"),
    },
    BuiltinRuleAsset {
        name: "empty-line-after-outer-attr",
        markdown_path: "builtin/rust/rs091-empty-line-after-outer-attr.md",
        query_path: "builtin/rust/rs091-empty-line-after-outer-attr.df",
        markdown: include_str!("../builtin/rust/rs091-empty-line-after-outer-attr.md"),
        query: include_str!("../builtin/rust/rs091-empty-line-after-outer-attr.df"),
    },
    BuiltinRuleAsset {
        name: "empty-loop",
        markdown_path: "builtin/rust/rs092-empty-loop.md",
        query_path: "builtin/rust/rs092-empty-loop.df",
        markdown: include_str!("../builtin/rust/rs092-empty-loop.md"),
        query: include_str!("../builtin/rust/rs092-empty-loop.df"),
    },
    BuiltinRuleAsset {
        name: "four-forward-slashes",
        markdown_path: "builtin/rust/rs095-four-forward-slashes.md",
        query_path: "builtin/rust/rs095-four-forward-slashes.df",
        markdown: include_str!("../builtin/rust/rs095-four-forward-slashes.md"),
        query: include_str!("../builtin/rust/rs095-four-forward-slashes.df"),
    },
    BuiltinRuleAsset {
        name: "from-raw-with-void-ptr",
        markdown_path: "builtin/rust/rs096-from-raw-with-void-ptr.md",
        query_path: "builtin/rust/rs096-from-raw-with-void-ptr.df",
        markdown: include_str!("../builtin/rust/rs096-from-raw-with-void-ptr.md"),
        query: include_str!("../builtin/rust/rs096-from-raw-with-void-ptr.df"),
    },
    BuiltinRuleAsset {
        name: "join-absolute-paths",
        markdown_path: "builtin/rust/rs101-join-absolute-paths.md",
        query_path: "builtin/rust/rs101-join-absolute-paths.df",
        markdown: include_str!("../builtin/rust/rs101-join-absolute-paths.md"),
        query: include_str!("../builtin/rust/rs101-join-absolute-paths.df"),
    },
    BuiltinRuleAsset {
        name: "let-underscore-future",
        markdown_path: "builtin/rust/rs102-let-underscore-future.md",
        query_path: "builtin/rust/rs102-let-underscore-future.df",
        markdown: include_str!("../builtin/rust/rs102-let-underscore-future.md"),
        query: include_str!("../builtin/rust/rs102-let-underscore-future.df"),
    },
    BuiltinRuleAsset {
        name: "manual-unwrap-or-default",
        markdown_path: "builtin/rust/rs105-manual-unwrap-or-default.md",
        query_path: "builtin/rust/rs105-manual-unwrap-or-default.df",
        markdown: include_str!("../builtin/rust/rs105-manual-unwrap-or-default.md"),
        query: include_str!("../builtin/rust/rs105-manual-unwrap-or-default.df"),
    },
    BuiltinRuleAsset {
        name: "misrefactored-assign-op",
        markdown_path: "builtin/rust/rs107-misrefactored-assign-op.md",
        query_path: "builtin/rust/rs107-misrefactored-assign-op.df",
        markdown: include_str!("../builtin/rust/rs107-misrefactored-assign-op.md"),
        query: include_str!("../builtin/rust/rs107-misrefactored-assign-op.df"),
    },
    BuiltinRuleAsset {
        name: "multi-assignments",
        markdown_path: "builtin/rust/rs109-multi-assignments.md",
        query_path: "builtin/rust/rs109-multi-assignments.df",
        markdown: include_str!("../builtin/rust/rs109-multi-assignments.md"),
        query: include_str!("../builtin/rust/rs109-multi-assignments.df"),
    },
    BuiltinRuleAsset {
        name: "mut-range-bound",
        markdown_path: "builtin/rust/rs111-mut-range-bound.md",
        query_path: "builtin/rust/rs111-mut-range-bound.df",
        markdown: include_str!("../builtin/rust/rs111-mut-range-bound.md"),
        query: include_str!("../builtin/rust/rs111-mut-range-bound.df"),
    },
    BuiltinRuleAsset {
        name: "needless-character-iteration",
        markdown_path: "builtin/rust/rs113-needless-character-iteration.md",
        query_path: "builtin/rust/rs113-needless-character-iteration.df",
        markdown: include_str!("../builtin/rust/rs113-needless-character-iteration.md"),
        query: include_str!("../builtin/rust/rs113-needless-character-iteration.df"),
    },
    BuiltinRuleAsset {
        name: "no-effect-replace",
        markdown_path: "builtin/rust/rs115-no-effect-replace.md",
        query_path: "builtin/rust/rs115-no-effect-replace.df",
        markdown: include_str!("../builtin/rust/rs115-no-effect-replace.md"),
        query: include_str!("../builtin/rust/rs115-no-effect-replace.df"),
    },
    BuiltinRuleAsset {
        name: "octal-escapes",
        markdown_path: "builtin/rust/rs118-octal-escapes.md",
        query_path: "builtin/rust/rs118-octal-escapes.df",
        markdown: include_str!("../builtin/rust/rs118-octal-escapes.md"),
        query: include_str!("../builtin/rust/rs118-octal-escapes.df"),
    },
    BuiltinRuleAsset {
        name: "path-ends-with-ext",
        markdown_path: "builtin/rust/rs119-path-ends-with-ext.md",
        query_path: "builtin/rust/rs119-path-ends-with-ext.df",
        markdown: include_str!("../builtin/rust/rs119-path-ends-with-ext.md"),
        query: include_str!("../builtin/rust/rs119-path-ends-with-ext.df"),
    },
    BuiltinRuleAsset {
        name: "permissions-set-readonly-false",
        markdown_path: "builtin/rust/rs120-permissions-set-readonly-false.md",
        query_path: "builtin/rust/rs120-permissions-set-readonly-false.df",
        markdown: include_str!("../builtin/rust/rs120-permissions-set-readonly-false.md"),
        query: include_str!("../builtin/rust/rs120-permissions-set-readonly-false.df"),
    },
    BuiltinRuleAsset {
        name: "pointers-in-nomem-asm-block",
        markdown_path: "builtin/rust/rs121-pointers-in-nomem-asm-block.md",
        query_path: "builtin/rust/rs121-pointers-in-nomem-asm-block.df",
        markdown: include_str!("../builtin/rust/rs121-pointers-in-nomem-asm-block.md"),
        query: include_str!("../builtin/rust/rs121-pointers-in-nomem-asm-block.df"),
    },
    BuiltinRuleAsset {
        name: "rc-clone-in-vec-init",
        markdown_path: "builtin/rust/rs123-rc-clone-in-vec-init.md",
        query_path: "builtin/rust/rs123-rc-clone-in-vec-init.df",
        markdown: include_str!("../builtin/rust/rs123-rc-clone-in-vec-init.md"),
        query: include_str!("../builtin/rust/rs123-rc-clone-in-vec-init.df"),
    },
    BuiltinRuleAsset {
        name: "repeat-vec-with-capacity",
        markdown_path: "builtin/rust/rs125-repeat-vec-with-capacity.md",
        query_path: "builtin/rust/rs125-repeat-vec-with-capacity.df",
        markdown: include_str!("../builtin/rust/rs125-repeat-vec-with-capacity.md"),
        query: include_str!("../builtin/rust/rs125-repeat-vec-with-capacity.df"),
    },
    BuiltinRuleAsset {
        name: "repr-packed-without-abi",
        markdown_path: "builtin/rust/rs126-repr-packed-without-abi.md",
        query_path: "builtin/rust/rs126-repr-packed-without-abi.df",
        markdown: include_str!("../builtin/rust/rs126-repr-packed-without-abi.md"),
        query: include_str!("../builtin/rust/rs126-repr-packed-without-abi.df"),
    },
    BuiltinRuleAsset {
        name: "single-range-in-vec-init",
        markdown_path: "builtin/rust/rs127-single-range-in-vec-init.md",
        query_path: "builtin/rust/rs127-single-range-in-vec-init.df",
        markdown: include_str!("../builtin/rust/rs127-single-range-in-vec-init.md"),
        query: include_str!("../builtin/rust/rs127-single-range-in-vec-init.df"),
    },
    BuiltinRuleAsset {
        name: "size-of-ref",
        markdown_path: "builtin/rust/rs128-size-of-ref.md",
        query_path: "builtin/rust/rs128-size-of-ref.df",
        markdown: include_str!("../builtin/rust/rs128-size-of-ref.md"),
        query: include_str!("../builtin/rust/rs128-size-of-ref.df"),
    },
    BuiltinRuleAsset {
        name: "suspicious-assignment-formatting",
        markdown_path: "builtin/rust/rs130-suspicious-assignment-formatting.md",
        query_path: "builtin/rust/rs130-suspicious-assignment-formatting.df",
        markdown: include_str!("../builtin/rust/rs130-suspicious-assignment-formatting.md"),
        query: include_str!("../builtin/rust/rs130-suspicious-assignment-formatting.df"),
    },
    BuiltinRuleAsset {
        name: "suspicious-command-arg-space",
        markdown_path: "builtin/rust/rs131-suspicious-command-arg-space.md",
        query_path: "builtin/rust/rs131-suspicious-command-arg-space.df",
        markdown: include_str!("../builtin/rust/rs131-suspicious-command-arg-space.md"),
        query: include_str!("../builtin/rust/rs131-suspicious-command-arg-space.df"),
    },
    BuiltinRuleAsset {
        name: "suspicious-doc-comments",
        markdown_path: "builtin/rust/rs132-suspicious-doc-comments.md",
        query_path: "builtin/rust/rs132-suspicious-doc-comments.df",
        markdown: include_str!("../builtin/rust/rs132-suspicious-doc-comments.md"),
        query: include_str!("../builtin/rust/rs132-suspicious-doc-comments.df"),
    },
    BuiltinRuleAsset {
        name: "suspicious-else-formatting",
        markdown_path: "builtin/rust/rs133-suspicious-else-formatting.md",
        query_path: "builtin/rust/rs133-suspicious-else-formatting.df",
        markdown: include_str!("../builtin/rust/rs133-suspicious-else-formatting.md"),
        query: include_str!("../builtin/rust/rs133-suspicious-else-formatting.df"),
    },
    BuiltinRuleAsset {
        name: "suspicious-unary-op-formatting",
        markdown_path: "builtin/rust/rs138-suspicious-unary-op-formatting.md",
        query_path: "builtin/rust/rs138-suspicious-unary-op-formatting.df",
        markdown: include_str!("../builtin/rust/rs138-suspicious-unary-op-formatting.md"),
        query: include_str!("../builtin/rust/rs138-suspicious-unary-op-formatting.df"),
    },
    BuiltinRuleAsset {
        name: "swap-ptr-to-ref",
        markdown_path: "builtin/rust/rs139-swap-ptr-to-ref.md",
        query_path: "builtin/rust/rs139-swap-ptr-to-ref.df",
        markdown: include_str!("../builtin/rust/rs139-swap-ptr-to-ref.md"),
        query: include_str!("../builtin/rust/rs139-swap-ptr-to-ref.df"),
    },
    BuiltinRuleAsset {
        name: "no-os-getenv",
        markdown_path: "builtin/python/py003-no-os-getenv.md",
        query_path: "builtin/python/py003-no-os-getenv.df",
        markdown: include_str!("../builtin/python/py003-no-os-getenv.md"),
        query: include_str!("../builtin/python/py003-no-os-getenv.df"),
    },
    BuiltinRuleAsset {
        name: "no-bare-except",
        markdown_path: "builtin/python/py004-no-bare-except.md",
        query_path: "builtin/python/py004-no-bare-except.df",
        markdown: include_str!("../builtin/python/py004-no-bare-except.md"),
        query: include_str!("../builtin/python/py004-no-bare-except.df"),
    },
    BuiltinRuleAsset {
        name: "none-comparison",
        markdown_path: "builtin/python/py005-none-comparison.md",
        query_path: "builtin/python/py005-none-comparison.df",
        markdown: include_str!("../builtin/python/py005-none-comparison.md"),
        query: include_str!("../builtin/python/py005-none-comparison.df"),
    },
    BuiltinRuleAsset {
        name: "true-false-comparison",
        markdown_path: "builtin/python/py006-true-false-comparison.md",
        query_path: "builtin/python/py006-true-false-comparison.df",
        markdown: include_str!("../builtin/python/py006-true-false-comparison.md"),
        query: include_str!("../builtin/python/py006-true-false-comparison.df"),
    },
    BuiltinRuleAsset {
        name: "not-in-test",
        markdown_path: "builtin/python/py007-not-in-test.md",
        query_path: "builtin/python/py007-not-in-test.df",
        markdown: include_str!("../builtin/python/py007-not-in-test.md"),
        query: include_str!("../builtin/python/py007-not-in-test.df"),
    },
    BuiltinRuleAsset {
        name: "not-is-test",
        markdown_path: "builtin/python/py008-not-is-test.md",
        query_path: "builtin/python/py008-not-is-test.df",
        markdown: include_str!("../builtin/python/py008-not-is-test.md"),
        query: include_str!("../builtin/python/py008-not-is-test.df"),
    },
    BuiltinRuleAsset {
        name: "type-comparison",
        markdown_path: "builtin/python/py009-type-comparison.md",
        query_path: "builtin/python/py009-type-comparison.df",
        markdown: include_str!("../builtin/python/py009-type-comparison.md"),
        query: include_str!("../builtin/python/py009-type-comparison.df"),
    },
    BuiltinRuleAsset {
        name: "lambda-assignment",
        markdown_path: "builtin/python/py010-lambda-assignment.md",
        query_path: "builtin/python/py010-lambda-assignment.df",
        markdown: include_str!("../builtin/python/py010-lambda-assignment.md"),
        query: include_str!("../builtin/python/py010-lambda-assignment.df"),
    },
    BuiltinRuleAsset {
        name: "invalid-escape-sequence",
        markdown_path: "builtin/python/py012-invalid-escape-sequence.md",
        query_path: "builtin/python/py012-invalid-escape-sequence.df",
        markdown: include_str!("../builtin/python/py012-invalid-escape-sequence.md"),
        query: include_str!("../builtin/python/py012-invalid-escape-sequence.df"),
    },
    BuiltinRuleAsset {
        name: "f-string-missing-placeholders",
        markdown_path: "builtin/python/py014-f-string-missing-placeholders.md",
        query_path: "builtin/python/py014-f-string-missing-placeholders.df",
        markdown: include_str!("../builtin/python/py014-f-string-missing-placeholders.md"),
        query: include_str!("../builtin/python/py014-f-string-missing-placeholders.df"),
    },
    BuiltinRuleAsset {
        name: "multi-value-repeated-key-literal",
        markdown_path: "builtin/python/py015-multi-value-repeated-key-literal.md",
        query_path: "builtin/python/py015-multi-value-repeated-key-literal.df",
        markdown: include_str!("../builtin/python/py015-multi-value-repeated-key-literal.md"),
        query: include_str!("../builtin/python/py015-multi-value-repeated-key-literal.df"),
    },
    BuiltinRuleAsset {
        name: "assert-tuple",
        markdown_path: "builtin/python/py016-assert-tuple.md",
        query_path: "builtin/python/py016-assert-tuple.df",
        markdown: include_str!("../builtin/python/py016-assert-tuple.md"),
        query: include_str!("../builtin/python/py016-assert-tuple.df"),
    },
    BuiltinRuleAsset {
        name: "is-literal",
        markdown_path: "builtin/python/py017-is-literal.md",
        query_path: "builtin/python/py017-is-literal.df",
        markdown: include_str!("../builtin/python/py017-is-literal.md"),
        query: include_str!("../builtin/python/py017-is-literal.df"),
    },
    BuiltinRuleAsset {
        name: "if-tuple",
        markdown_path: "builtin/python/py019-if-tuple.md",
        query_path: "builtin/python/py019-if-tuple.df",
        markdown: include_str!("../builtin/python/py019-if-tuple.md"),
        query: include_str!("../builtin/python/py019-if-tuple.df"),
    },
    BuiltinRuleAsset {
        name: "break-outside-loop",
        markdown_path: "builtin/python/py020-break-outside-loop.md",
        query_path: "builtin/python/py020-break-outside-loop.df",
        markdown: include_str!("../builtin/python/py020-break-outside-loop.md"),
        query: include_str!("../builtin/python/py020-break-outside-loop.df"),
    },
    BuiltinRuleAsset {
        name: "continue-outside-loop",
        markdown_path: "builtin/python/py021-continue-outside-loop.md",
        query_path: "builtin/python/py021-continue-outside-loop.df",
        markdown: include_str!("../builtin/python/py021-continue-outside-loop.md"),
        query: include_str!("../builtin/python/py021-continue-outside-loop.df"),
    },
    BuiltinRuleAsset {
        name: "yield-outside-function",
        markdown_path: "builtin/python/py022-yield-outside-function.md",
        query_path: "builtin/python/py022-yield-outside-function.df",
        markdown: include_str!("../builtin/python/py022-yield-outside-function.md"),
        query: include_str!("../builtin/python/py022-yield-outside-function.df"),
    },
    BuiltinRuleAsset {
        name: "return-outside-function",
        markdown_path: "builtin/python/py023-return-outside-function.md",
        query_path: "builtin/python/py023-return-outside-function.df",
        markdown: include_str!("../builtin/python/py023-return-outside-function.md"),
        query: include_str!("../builtin/python/py023-return-outside-function.df"),
    },
    BuiltinRuleAsset {
        name: "default-except-not-last",
        markdown_path: "builtin/python/py024-default-except-not-last.md",
        query_path: "builtin/python/py024-default-except-not-last.df",
        markdown: include_str!("../builtin/python/py024-default-except-not-last.md"),
        query: include_str!("../builtin/python/py024-default-except-not-last.df"),
    },
    BuiltinRuleAsset {
        name: "raise-not-implemented",
        markdown_path: "builtin/python/py025-raise-not-implemented.md",
        query_path: "builtin/python/py025-raise-not-implemented.df",
        markdown: include_str!("../builtin/python/py025-raise-not-implemented.md"),
        query: include_str!("../builtin/python/py025-raise-not-implemented.df"),
    },
    BuiltinRuleAsset {
        name: "return-in-init",
        markdown_path: "builtin/python/py026-return-in-init.md",
        query_path: "builtin/python/py026-return-in-init.df",
        markdown: include_str!("../builtin/python/py026-return-in-init.md"),
        query: include_str!("../builtin/python/py026-return-in-init.df"),
    },
    BuiltinRuleAsset {
        name: "nonlocal-and-global",
        markdown_path: "builtin/python/py027-nonlocal-and-global.md",
        query_path: "builtin/python/py027-nonlocal-and-global.df",
        markdown: include_str!("../builtin/python/py027-nonlocal-and-global.md"),
        query: include_str!("../builtin/python/py027-nonlocal-and-global.df"),
    },
    BuiltinRuleAsset {
        name: "continue-in-finally",
        markdown_path: "builtin/python/py028-continue-in-finally.md",
        query_path: "builtin/python/py028-continue-in-finally.df",
        markdown: include_str!("../builtin/python/py028-continue-in-finally.md"),
        query: include_str!("../builtin/python/py028-continue-in-finally.df"),
    },
    BuiltinRuleAsset {
        name: "duplicate-bases",
        markdown_path: "builtin/python/py029-duplicate-bases.md",
        query_path: "builtin/python/py029-duplicate-bases.df",
        markdown: include_str!("../builtin/python/py029-duplicate-bases.md"),
        query: include_str!("../builtin/python/py029-duplicate-bases.df"),
    },
    BuiltinRuleAsset {
        name: "invalid-all-object",
        markdown_path: "builtin/python/py030-invalid-all-object.md",
        query_path: "builtin/python/py030-invalid-all-object.df",
        markdown: include_str!("../builtin/python/py030-invalid-all-object.md"),
        query: include_str!("../builtin/python/py030-invalid-all-object.df"),
    },
    BuiltinRuleAsset {
        name: "invalid-all-format",
        markdown_path: "builtin/python/py031-invalid-all-format.md",
        query_path: "builtin/python/py031-invalid-all-format.df",
        markdown: include_str!("../builtin/python/py031-invalid-all-format.md"),
        query: include_str!("../builtin/python/py031-invalid-all-format.df"),
    },
    BuiltinRuleAsset {
        name: "misplaced-bare-raise",
        markdown_path: "builtin/python/py032-misplaced-bare-raise.md",
        query_path: "builtin/python/py032-misplaced-bare-raise.df",
        markdown: include_str!("../builtin/python/py032-misplaced-bare-raise.md"),
        query: include_str!("../builtin/python/py032-misplaced-bare-raise.df"),
    },
];

pub const RULE_AUTHORING_GUIDE: &str = r#"lintbook custom rules are Rust-only in this version.

Rule files are same-stem pairs:
- .lintbook/rules/<slug>.md contains minimal frontmatter plus human-readable prose.
- .lintbook/gen/<slug>.df contains one generated Datafox query set.

Required Markdown frontmatter:
- id: stable rule id, for example "rust.no-dbg"
- lang: "rust"

lintbook derives the violation message from the first prose paragraph and uses `Node` as the primary query variable. Missing .df files are treated as incomplete rules. Do not put Markdown fences, comments, or prose in .df files; write only Datafox queries.

Datafox query grammar:
- query_set ::= query (";" query)* [";"]
- query ::= clause ("," clause)*
- clause ::= atom | builtin | "!" atom
- atom ::= predicate "(" [term ("," term)*] ")"
- builtin ::= term ("=" | "!=" | ">" | ">=" | "<" | "<=") term
- builtin ::= ("contains" | "startsWith" | "endsWith" | "matchesRegex" | "notContains" | "notStartsWith" | "notEndsWith" | "notMatchesRegex" | "before" | "after") "(" term "," term ")"
- term ::= Variable | "_" | integer | string | bare-lowercase-constant | single-quoted-string

Datafox syntax notes:
- Variables start with an uppercase ASCII letter, for example `Node`, `Text`, `StartLine`.
- Lowercase bare identifiers in term position are string constants, not variables.
- `_` is a wildcard and does not bind.
- Strings use double quotes. Single quotes are accepted for quoted strings/predicates.
- Predicate names start with an ASCII letter and may contain letters, digits, `:`, `_`, `-`, or `?`.
- Use `;` to put several queries in one .df file. A violation is emitted for every query result.
- Negated atom clauses and builtin clauses must come after facts that bind all variables they use.

Available Rust facts:
- node(Node, Kind, StartLine, StartColumn, EndLine, EndColumn)
- span(Node, StartByte, EndByte)
- location(Entity, Line, Column)
- text(Node, Text)
- trimmedText(Node, TrimmedText)
- lowerText(Node, LowercaseText)
- collapsedText(Node, WhitespaceCollapsedText)
- literal(Node, Kind, RawText, NormalizedText)
- intLiteralValue(Node, Value)
- child(Parent, Child, Index)
- argument(ArgumentsNode, ArgumentNode, Index)
- statementExpression(StatementNode, ExpressionNode)
- assignment(Node, Left, Right)
- comparison(Node, Left, Operator, Right)
- rangeBounds(Node, Left, Right)
- unitLike(Node)
- extremeValue(Node)
- moduleDecl(Node, Name)
- attributeName(AttributeNode, Name)
- attributeOf(TargetNode, AttributeNode, Name)
- mistypedLiteralSuffix(Node)
- possibleMissingComma(Node)
- invisibleCharacter(Node, Name, Codepoint)
- parent(Child, Parent)
- field(Parent, FieldName, Child)
- named(Node)
- descendant(Ancestor, Descendant)
- nextSibling(Left, Right)
- previousSibling(Right, Left)
- nextCodeSibling(Left, Right)
- sibling(Parent, Left, Right)
- lineGap(Left, Right, BlankLineCount)
- line(Line, LineNumber, Text, StartByte, EndByte)
- nextLine(Line, NextLine)
- previousLine(NextLine, Line)

Fact semantics:
- `Node`, `Parent`, `Child`, `Ancestor`, and `Descendant` are integer tree-sitter node ids.
- `Line` is a synthetic negative integer id for one source line.
- `Kind` and `FieldName` are tree-sitter strings.
- Line and column values are 1-based integers.
- `text(Node, Text)` stores the source text covered by that node.
- `trimmedText(Node, TrimmedText)` stores node text with leading and trailing whitespace removed.
- `lowerText(Node, LowercaseText)` stores ASCII-lowercased node text for case-insensitive string checks.
- `collapsedText(Node, WhitespaceCollapsedText)` collapses whitespace runs to one ASCII space.
- `literal(Node, Kind, RawText, NormalizedText)` stores classified literal text for tree-sitter literal nodes.
- `intLiteralValue(Node, Value)` parses integer literal values after removing underscores and Rust integer suffixes.
- `child` is direct parent-to-child with zero-based child index.
- `argument` is direct argument-list-to-argument with zero-based argument index.
- `statementExpression` links an expression statement to its direct expression child.
- `assignment`, `comparison`, and `rangeBounds` expose direct operand nodes for common Rust expression shapes.
- `unitLike`, `extremeValue`, `mistypedLiteralSuffix`, `possibleMissingComma`, and `invisibleCharacter` are derived helper facts for source-shape checks that are awkward to express as pure joins.
- `moduleDecl`, `attributeName`, and `attributeOf` expose Rust module and attribute ownership relationships.
- `parent` is direct child-to-parent.
- `field` is direct parent-to-child for named tree-sitter fields.
- `descendant` is transitive ancestor-to-descendant.
- `nextSibling`, `previousSibling`, and `sibling` are direct adjacent sibling relationships.
- `nextCodeSibling` is direct adjacent sibling order after skipping comments and anonymous punctuation.
- `lineGap(Left, Right, BlankLineCount)` counts blank physical lines between adjacent sibling nodes.
- `nextLine` and `previousLine` are direct adjacent source-line relationships.

Example rules:
- dbg macro calls:
  node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")
- functions named main:
  node(Node, "function_item", _, _, _, _), field(Node, "name", Name), text(Name, "main")
- multiline call expressions:
  node(Node, "call_expression", StartLine, _, EndLine, _), EndLine > StartLine
- todo macro calls inside functions:
  node(Node, "function_item", _, _, _, _), descendant(Node, Macro), node(Macro, "macro_invocation", _, _, _, _), text(Macro, Text), contains(Text, "todo!")

Testing workflow:
- Use the Markdown intent first. If the rule includes examples, turn those into positive and negative checks.
- If examples are not provided, create at least one minimal Rust snippet that should match and one that should not match.
- Inspect tree-sitter node kinds and fields with `lintbook dump-ast --lang rust <path>`.
- You can inspect a tiny snippet through stdin with `printf 'fn main() { dbg!(1); }\n' | lintbook dump-ast --lang rust`.
- After writing the .df file, run `lintbook compile` without `--agent`.
- Verify a positive example with `lintbook check --output json <positive.rs>`. A matching lint exits nonzero; inspect stdout and confirm the rule id appears.
- Verify a negative example with `lintbook check --output json <negative.rs>` and confirm it exits zero with no violation for the rule id.
- Stop once the rule compiles successfully and the focused positive/negative checks behave as expected.
- Do not try to fix every repository file that the new rule reports.
- Prefer temporary example files outside the repository. If sandboxing requires repository-local files, use a clearly temporary path and remove it before finishing.
"#;

#[derive(Debug, Clone, Serialize)]
pub struct CompileReport {
    pub compiled: Vec<String>,
    pub skipped_incomplete: Vec<IncompleteRule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncompleteRule {
    pub id: String,
    pub markdown_path: PathBuf,
    pub query_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuiltinRuleInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledRule {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub language: String,
    pub message_template: String,
    pub primary: String,
    pub markdown_sha256: String,
    pub query_sha256: String,
    pub queries: Vec<Query>,
}

#[derive(Debug)]
struct RuleFrontmatter {
    id: String,
    name: String,
    lang: String,
    message: String,
    primary: String,
}

#[derive(Debug)]
struct RuleSource {
    metadata: RuleFrontmatter,
    markdown_path: PathBuf,
    query_path: PathBuf,
    markdown_source: String,
    query_source: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct BuiltinRuleAsset {
    name: &'static str,
    markdown_path: &'static str,
    query_path: &'static str,
    markdown: &'static str,
    query: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct NodeLocation {
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedFactSet {
    schema_version: u32,
    language: String,
    source_sha256: String,
    predicate_fingerprint: String,
    storage: InMemoryStorage,
    locations: BTreeMap<i64, NodeLocation>,
}

impl CachedFactSet {
    fn into_runtime(self) -> (InMemoryStorage, HashMap<i64, NodeLocation>) {
        (self.storage, self.locations.into_iter().collect())
    }
}

pub fn rules_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(LINTBOOK_DIR).join(RULES_DIR)
}

pub fn gen_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(LINTBOOK_DIR).join(GEN_DIR)
}

pub fn cache_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(LINTBOOK_DIR).join(CACHE_DIR)
}

fn prepared_query_cache_dir(repo_root: &Path) -> PathBuf {
    cache_dir(repo_root)
        .join(PREPARED_QUERIES_DIR)
        .join(format!(
            "datafox-v{}",
            datafox::PREPARED_QUERY_FORMAT_VERSION
        ))
        .join(format!("bincode-v{BINCODE_CACHE_FORMAT_VERSION}"))
}

#[derive(Clone)]
struct FilePreparedQueryStorage {
    memory: InMemoryPreparedQueryStorage,
    directory: PathBuf,
}

impl FilePreparedQueryStorage {
    fn new(repo_root: &Path) -> Self {
        Self {
            memory: InMemoryPreparedQueryStorage::unbounded(),
            directory: prepared_query_cache_dir(repo_root),
        }
    }

    fn path_for(&self, key: &PreparedQueryKey) -> datafox::Result<PathBuf> {
        let encoded = bincode_encode(key).map_err(prepared_query_storage_error)?;
        Ok(self
            .directory
            .join(format!("{}.bin", sha256_hex_bytes(&encoded))))
    }
}

impl PreparedQueryStorage for FilePreparedQueryStorage {
    fn get(&self, key: &PreparedQueryKey) -> datafox::Result<Option<Arc<PreparedQuery>>> {
        if let Some(prepared) = self.memory.get(key)? {
            return Ok(Some(prepared));
        }

        let path = self.path_for(key)?;
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(None),
        };
        let prepared: PreparedQuery = match bincode_decode(&bytes) {
            Ok(prepared) => prepared,
            Err(_) => {
                let _ = fs::remove_file(&path);
                return Ok(None);
            }
        };
        if prepared.validate().is_err() {
            let _ = fs::remove_file(&path);
            return Ok(None);
        }

        let prepared = Arc::new(prepared);
        self.memory.insert(key.clone(), Arc::clone(&prepared))?;
        Ok(Some(prepared))
    }

    fn insert(&self, key: PreparedQueryKey, prepared: Arc<PreparedQuery>) -> datafox::Result<()> {
        self.memory.insert(key.clone(), Arc::clone(&prepared))?;

        let path = self.path_for(&key)?;
        let encoded = bincode_encode(prepared.as_ref()).map_err(prepared_query_storage_error)?;
        let _ = write_prepared_query_cache_file(&path, &encoded);
        Ok(())
    }
}

fn write_prepared_query_cache_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension(format!("bin.tmp-{}", std::process::id()));
    if let Err(error) = fs::write(&temp_path, bytes).and_then(|()| fs::rename(&temp_path, path)) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    Ok(())
}

fn prepared_query_storage_error(error: impl std::fmt::Display) -> datafox::Error {
    datafox::Error::PreparedQueryStorage {
        message: error.to_string(),
    }
}

fn bincode_encode<T: Serialize>(
    value: &T,
) -> std::result::Result<Vec<u8>, bincode::error::EncodeError> {
    bincode::serde::encode_to_vec(value, bincode::config::legacy())
}

fn bincode_decode<T: DeserializeOwned>(
    bytes: &[u8],
) -> std::result::Result<T, bincode::error::DecodeError> {
    let (value, bytes_read) = std::panic::catch_unwind(|| {
        bincode::serde::decode_from_slice(bytes, bincode::config::legacy())
    })
    .map_err(|_| bincode::error::DecodeError::Other("bincode decode panicked"))??;
    if bytes_read == bytes.len() {
        Ok(value)
    } else {
        Err(bincode::error::DecodeError::Other("trailing bytes"))
    }
}

pub fn compiled_rule_path(repo_root: &Path, id: &str) -> PathBuf {
    gen_dir(repo_root).join(format!("{}.json", sanitize_file_stem(id)))
}

pub fn generated_query_path(repo_root: &Path, markdown_path: &Path) -> Result<PathBuf> {
    let stem = markdown_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| anyhow!("Invalid rule markdown path {}", markdown_path.display()))?;
    Ok(gen_dir(repo_root).join(format!("{stem}.df")))
}

pub fn compile_project(repo_root: &Path) -> Result<CompileReport> {
    let rules_dir = rules_dir(repo_root);
    let gen_dir = gen_dir(repo_root);

    if !rules_dir.exists() {
        bail!("No .lintbook/rules directory found. Run `lintbook setup` before compiling rules.");
    }

    fs::create_dir_all(&gen_dir)?;
    fs::write(gen_dir.join(".gitkeep"), "")?;

    let sources = read_rule_sources(repo_root, false)?;
    let mut seen_ids = HashSet::new();
    let mut expected_outputs = BTreeSet::new();
    let mut compiled = Vec::new();
    let mut skipped_incomplete = Vec::new();

    for mut source in sources {
        if !seen_ids.insert(source.metadata.id.clone()) {
            bail!("Duplicate lintbook rule id `{}`", source.metadata.id);
        }

        if source.query_source.is_none() {
            skipped_incomplete.push(source.incomplete_rule());
            continue;
        }

        normalize_query_source(&mut source)?;
        let compiled_rule = compile_source(&source)?;
        let output_path = compiled_rule_path(repo_root, &compiled_rule.id);
        expected_outputs.insert(output_path.clone());
        let json = serde_json::to_string_pretty(&compiled_rule)?;
        fs::write(&output_path, format!("{json}\n"))?;
        compiled.push(compiled_rule.id);
    }

    for entry in fs::read_dir(&gen_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json")
            && !expected_outputs.contains(&path)
        {
            fs::remove_file(path)?;
        }
    }

    Ok(CompileReport {
        compiled,
        skipped_incomplete,
    })
}

pub fn builtin_rule_infos() -> Result<Vec<BuiltinRuleInfo>> {
    let mut infos = Vec::new();
    for asset in BUILTIN_RULES {
        let metadata = parse_frontmatter_for_source(
            asset.markdown_path,
            asset.markdown,
            FrontmatterMode::Builtin,
        )?;
        infos.push(BuiltinRuleInfo {
            id: metadata.id,
            name: asset.name.to_string(),
            language: metadata.lang,
            description: metadata.message,
        });
    }
    Ok(infos)
}

pub fn compile_builtin_rules() -> Result<Vec<CompiledRule>> {
    BUILTIN_RULES
        .iter()
        .map(|asset| {
            let mut rule = compile_source_parts(
                asset.markdown_path,
                asset.query_path,
                asset.markdown,
                asset.query,
                FrontmatterMode::Builtin,
            )?;
            rule.name = asset.name.to_string();
            Ok(rule)
        })
        .collect()
}

fn normalize_query_source(source: &mut RuleSource) -> Result<()> {
    let query_source = source
        .query_source
        .as_ref()
        .expect("query source is present");
    let queries = datafox::parse_queries(query_source)
        .with_context(|| format!("Failed to parse {}", source.query_path.display()))?;
    let formatted = format!("{}\n", datafox::format_queries(&queries));

    if formatted != *query_source {
        fs::write(&source.query_path, &formatted)
            .with_context(|| format!("Failed to write {}", source.query_path.display()))?;
        source.query_source = Some(formatted);
    }

    Ok(())
}

pub fn load_compiled_rules(repo_root: &Path) -> Result<Vec<CompiledRule>> {
    let rules_dir = rules_dir(repo_root);
    if !rules_dir.exists() {
        return Ok(Vec::new());
    }

    let sources = read_rule_sources(repo_root, true)?;
    let mut rules = Vec::new();

    for source in sources {
        if source.query_source.is_none() {
            continue;
        }

        let markdown_hash = sha256_hex(&source.markdown_source);
        let query_source = source
            .query_source
            .as_ref()
            .expect("query source is present");
        let query_hash = sha256_hex(query_source);
        let path = compiled_rule_path(repo_root, &source.metadata.id);
        if !path.exists() {
            return Err(stale_error(&source.metadata.id, &path));
        }

        let compiled_source = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let compiled: CompiledRule = serde_json::from_str(&compiled_source)
            .map_err(|_| stale_error(&source.metadata.id, &path))?;

        if compiled.schema_version != SCHEMA_VERSION
            || compiled.markdown_sha256 != markdown_hash
            || compiled.query_sha256 != query_hash
            || compiled.id != source.metadata.id
            || compiled.name != source.metadata.name
            || compiled.language != source.metadata.lang
            || compiled.primary != source.metadata.primary
        {
            return Err(stale_error(&source.metadata.id, &path));
        }

        rules.push(compiled);
    }

    Ok(rules)
}

pub fn load_all_rules(repo_root: &Path, config: &LintbookConfig) -> Result<Vec<CompiledRule>> {
    let mut rules = compile_builtin_rules()?
        .into_iter()
        .filter(|rule| rule_enabled(config, rule))
        .collect::<Vec<_>>();

    rules.extend(
        load_compiled_rules(repo_root)?
            .into_iter()
            .filter(|rule| rule_enabled(config, rule)),
    );

    Ok(rules)
}

pub fn active_rule_languages(
    repo_root: &Path,
    config: &LintbookConfig,
) -> Result<HashSet<Grammar>> {
    let mut languages = HashSet::new();

    for info in builtin_rule_infos()? {
        if config.is_lint_enabled(&info.language, &info.name) {
            languages.insert(Grammar::from_name(&info.language)?);
        }
    }

    for rule in load_compiled_rules(repo_root)? {
        if rule_enabled(config, &rule) {
            languages.insert(Grammar::from_name(&rule.language)?);
        }
    }

    Ok(languages)
}

pub fn incomplete_rules(repo_root: &Path) -> Result<Vec<IncompleteRule>> {
    Ok(read_rule_sources(repo_root, true)?
        .into_iter()
        .filter(|source| source.query_source.is_none())
        .map(|source| source.incomplete_rule())
        .collect())
}

pub async fn run_generated_rules(
    repo_root: &Path,
    config: &LintbookConfig,
    results: &[LintResult<Grammar>],
) -> Result<BTreeMap<PathBuf, Vec<LintViolation>>> {
    run_generated_rules_with_profile(
        repo_root,
        config,
        results,
        GeneratedRuleEvaluationProfile::serial(),
    )
    .await
}

pub async fn run_generated_rules_with_profile(
    repo_root: &Path,
    config: &LintbookConfig,
    results: &[LintResult<Grammar>],
    evaluation_profile: GeneratedRuleEvaluationProfile,
) -> Result<BTreeMap<PathBuf, Vec<LintViolation>>> {
    let runner = Arc::new(GeneratedRuleRunner::new_with_profile(
        repo_root,
        config,
        evaluation_profile,
    )?);
    if runner.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut output = BTreeMap::new();
    let mut tasks = Vec::new();

    for result in results {
        if matches!(result.status, LintStatus::Skipped) || output.contains_key(&result.file_path) {
            continue;
        }

        let runner = Arc::clone(&runner);
        let result = result.clone();
        tasks.push(tokio::task::spawn_blocking(
            move || -> Result<Option<(PathBuf, Vec<LintViolation>)>> {
                let violations = runner.run_on_lint_result(&result)?;
                if violations.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some((result.file_path, violations)))
                }
            },
        ));
    }

    for task in tasks {
        if let Some((file_path, violations)) =
            task.await.context("Generated lint worker panicked")??
        {
            output.insert(file_path, violations);
        }
    }

    Ok(output)
}

#[derive(Clone)]
pub struct GeneratedRuleRunner {
    repo_root: PathBuf,
    rules_by_language: HashMap<Grammar, Arc<Vec<PreparedCompiledRule>>>,
    evaluation_profile: GeneratedRuleEvaluationProfile,
    datafox_environment: DatafoxEnvironment,
}

#[derive(Clone)]
struct PreparedCompiledRule {
    rule: CompiledRule,
    prepared_queries: Vec<Arc<PreparedQuery>>,
}

impl PreparedCompiledRule {
    fn new(rule: CompiledRule, datafox_environment: &DatafoxEnvironment) -> Result<Self> {
        let prepared_queries = rule
            .queries
            .iter()
            .map(|query| datafox_environment.prepare(query))
            .collect::<datafox::Result<Vec<_>>>()
            .with_context(|| format!("Failed to prepare generated rule `{}`", rule.id))?;

        Ok(Self {
            rule,
            prepared_queries,
        })
    }
}

fn prepare_compiled_rules(
    rules: impl IntoIterator<Item = CompiledRule>,
    datafox_environment: &DatafoxEnvironment,
) -> Result<Vec<PreparedCompiledRule>> {
    rules
        .into_iter()
        .map(|rule| PreparedCompiledRule::new(rule, datafox_environment))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedRuleEvaluationProfile {
    Serial,
    Parallel { seed_threshold: Option<usize> },
}

impl GeneratedRuleEvaluationProfile {
    pub fn serial() -> Self {
        Self::Serial
    }

    pub fn parallel() -> Self {
        Self::Parallel {
            seed_threshold: None,
        }
    }

    pub fn parallel_with_seed_threshold(seed_threshold: usize) -> Self {
        Self::Parallel {
            seed_threshold: Some(seed_threshold),
        }
    }

    fn client_for<'store>(
        &self,
        storage: &'store InMemoryStorage,
        datafox_environment: &DatafoxEnvironment,
    ) -> Result<DatafoxClient<'store>> {
        let config = DatafoxConfig::new(storage).with_environment(datafox_environment.clone());
        let config = match *self {
            Self::Serial => config.serial(),
            Self::Parallel { seed_threshold } => {
                let config = config.parallel();
                if let Some(seed_threshold) = seed_threshold {
                    config.seed_threshold(seed_threshold)
                } else {
                    config
                }
            }
        };

        DatafoxClient::new(config).context("Failed to build generated rule evaluator")
    }
}

impl GeneratedRuleRunner {
    pub fn new(repo_root: &Path, config: &LintbookConfig) -> Result<Self> {
        Self::new_with_profile(repo_root, config, GeneratedRuleEvaluationProfile::serial())
    }

    pub fn new_with_profile(
        repo_root: &Path,
        config: &LintbookConfig,
        evaluation_profile: GeneratedRuleEvaluationProfile,
    ) -> Result<Self> {
        let rules = load_all_rules(repo_root, config)?;
        let datafox_environment = DatafoxEnvironment::builder()
            .with_prepared_query_storage(FilePreparedQueryStorage::new(repo_root))
            .build();
        let mut rules_by_language: HashMap<Grammar, Vec<PreparedCompiledRule>> = HashMap::new();
        for rule in rules {
            let language = Grammar::from_name(&rule.language).with_context(|| {
                format!("Unsupported generated rule language `{}`", rule.language)
            })?;
            let mut prepared = prepare_compiled_rules([rule], &datafox_environment)?;
            let rule = prepared.pop().expect("one prepared rule");
            rules_by_language.entry(language).or_default().push(rule);
        }

        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            rules_by_language: rules_by_language
                .into_iter()
                .map(|(language, rules)| (language, Arc::new(rules)))
                .collect(),
            evaluation_profile,
            datafox_environment,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.rules_by_language.is_empty()
    }

    pub fn run_on_lint_result(&self, result: &LintResult<Grammar>) -> Result<Vec<LintViolation>> {
        if matches!(result.status, LintStatus::Skipped) {
            return Ok(Vec::new());
        }

        let Some(grammar) = result.language else {
            return Ok(Vec::new());
        };
        let Some(rules) = self.rules_by_language.get(&grammar) else {
            return Ok(Vec::new());
        };

        let source = match fs::read_to_string(&result.file_path) {
            Ok(source) => source,
            Err(_) => return Ok(Vec::new()),
        };

        run_rules_on_file_sync_with_profile(
            &self.repo_root,
            grammar,
            &source,
            rules,
            self.evaluation_profile,
            &self.datafox_environment,
        )
    }

    #[cfg(test)]
    fn prepared_query_count(&self) -> usize {
        self.rules_by_language
            .values()
            .flat_map(|rules| rules.iter())
            .map(|rule| rule.prepared_queries.len())
            .sum()
    }
}

fn stale_error(id: &str, path: &Path) -> anyhow::Error {
    anyhow!(
        "Generated lintbook rule `{}` is missing or stale at {}. Run `lintbook compile`.",
        id,
        path.display()
    )
}

fn read_rule_sources(repo_root: &Path, _allow_missing_query: bool) -> Result<Vec<RuleSource>> {
    let rules_dir = rules_dir(repo_root);
    let mut markdown_paths = fs::read_dir(&rules_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .filter(|path| path.file_stem().and_then(|stem| stem.to_str()) != Some("template"))
        .collect::<Vec<_>>();
    markdown_paths.sort();

    let mut sources = Vec::new();
    for markdown_path in markdown_paths {
        let markdown_source = fs::read_to_string(&markdown_path)
            .with_context(|| format!("Failed to read {}", markdown_path.display()))?;
        let metadata = parse_frontmatter(&markdown_path, &markdown_source)?;
        let query_path = generated_query_path(repo_root, &markdown_path)?;
        let query_source = match fs::read_to_string(&query_path) {
            Ok(source) => Some(source),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to read {}", query_path.display()));
            }
        };

        sources.push(RuleSource {
            metadata,
            markdown_path,
            query_path,
            markdown_source,
            query_source,
        });
    }

    Ok(sources)
}

impl RuleSource {
    fn incomplete_rule(self) -> IncompleteRule {
        IncompleteRule {
            id: self.metadata.id,
            markdown_path: self.markdown_path,
            query_path: self.query_path,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FrontmatterMode {
    Project,
    Builtin,
}

fn parse_frontmatter(path: &Path, source: &str) -> Result<RuleFrontmatter> {
    parse_frontmatter_for_source(
        &path.display().to_string(),
        source,
        FrontmatterMode::Project,
    )
}

fn parse_frontmatter_for_source(
    path_label: &str,
    source: &str,
    mode: FrontmatterMode,
) -> Result<RuleFrontmatter> {
    let Some(rest) = source.strip_prefix("---\n") else {
        bail!("{path_label} must start with frontmatter delimited by ---");
    };
    let Some((frontmatter, body)) = rest.split_once("\n---") else {
        bail!("{path_label} has unterminated frontmatter");
    };

    let mut id = None;
    let mut name = None;
    let mut lang = None;
    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            bail!("Invalid frontmatter line in {path_label}: {line}");
        };
        let value = trim_frontmatter_value(value.trim());
        match key.trim() {
            "id" => id = Some(value.to_string()),
            "name" => name = Some(value.to_string()),
            "lang" => lang = Some(value.to_string()),
            "language" => lang = Some(value.to_string()),
            _ => {}
        }
    }

    let id = id.ok_or_else(|| anyhow!("{path_label} is missing frontmatter key `id`"))?;
    let lang = lang.ok_or_else(|| anyhow!("{path_label} is missing frontmatter key `lang`"))?;
    let name = name.unwrap_or_else(|| id.clone());
    let metadata = RuleFrontmatter {
        message: message_from_body(&id, body),
        primary: "Node".to_string(),
        id,
        name,
        lang,
    };

    if metadata.id.trim().is_empty() {
        bail!("{path_label} has an empty rule id");
    }
    if metadata.name.trim().is_empty() {
        bail!("{path_label} has an empty rule name");
    }
    if matches!(mode, FrontmatterMode::Project) && metadata.lang != "rust" {
        bail!(
            "{} uses unsupported custom rule language `{}`; v1 supports rust only",
            path_label,
            metadata.lang
        );
    }
    if metadata.message.trim().is_empty() {
        bail!("{path_label} has an empty rule body");
    }
    if metadata.primary.trim().is_empty() {
        bail!("{path_label} has an empty primary variable");
    }

    Ok(metadata)
}

fn trim_frontmatter_value(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn message_from_body(id: &str, body: &str) -> String {
    let body = body.strip_prefix('\n').unwrap_or(body);
    let mut lines = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        lines.push(line);
    }

    if lines.is_empty() {
        format!("Rule `{id}` matched")
    } else {
        lines.join(" ")
    }
}

fn compile_source(source: &RuleSource) -> Result<CompiledRule> {
    let query_source = source
        .query_source
        .as_ref()
        .expect("query source is present");
    compile_source_parts(
        &source.markdown_path.display().to_string(),
        &source.query_path.display().to_string(),
        &source.markdown_source,
        query_source,
        FrontmatterMode::Project,
    )
}

fn compile_source_parts(
    markdown_path: &str,
    query_path: &str,
    markdown_source: &str,
    query_source: &str,
    mode: FrontmatterMode,
) -> Result<CompiledRule> {
    let metadata = parse_frontmatter_for_source(markdown_path, markdown_source, mode)?;
    let queries = datafox::parse_queries(query_source)
        .with_context(|| format!("Failed to parse {query_path}"))?;
    validate_queries(&metadata, &queries)
        .with_context(|| format!("Invalid query in {query_path}"))?;

    Ok(CompiledRule {
        schema_version: SCHEMA_VERSION,
        id: metadata.id,
        name: metadata.name,
        language: metadata.lang,
        message_template: metadata.message,
        primary: metadata.primary,
        markdown_sha256: sha256_hex(markdown_source),
        query_sha256: sha256_hex(query_source),
        queries,
    })
}

fn validate_queries(metadata: &RuleFrontmatter, queries: &[Query]) -> Result<()> {
    if queries.is_empty() {
        bail!("rule must contain at least one query");
    }
    for query in queries {
        validate_query(metadata, query)?;
    }
    Ok(())
}

fn validate_query(metadata: &RuleFrontmatter, query: &Query) -> Result<()> {
    let mut bound = HashSet::new();

    for clause in query.clauses() {
        match clause {
            Clause::Atom(atom) => {
                for variable in atom.variables() {
                    bound.insert(variable.to_string());
                }
            }
            Clause::Negated(atom) => {
                for variable in atom.variables() {
                    if !bound.contains(variable) {
                        bail!(
                            "negated predicate `{}` uses `{}` before that variable is bound by an earlier fact clause",
                            atom.predicate,
                            variable
                        );
                    }
                }
            }
            Clause::Builtin { name, args } => {
                for variable in args.iter().flat_map(Term::variables) {
                    if !bound.contains(variable) {
                        bail!(
                            "builtin `{}` uses `{}` before that variable is bound by an earlier fact clause",
                            name,
                            variable
                        );
                    }
                }
            }
        }
    }

    if !bound.contains(&metadata.primary) {
        bail!(
            "primary variable `{}` is not bound by the query",
            metadata.primary
        );
    }

    Ok(())
}

#[cfg(test)]
fn run_rules_on_file_sync(
    repo_root: &Path,
    grammar: Grammar,
    source: &str,
    rules: &[CompiledRule],
) -> Result<Vec<LintViolation>> {
    let datafox_environment = DatafoxEnvironment::builder()
        .with_prepared_query_storage(InMemoryPreparedQueryStorage::unbounded())
        .build();
    let rules = prepare_compiled_rules(rules.iter().cloned(), &datafox_environment)?;
    run_rules_on_file_sync_with_profile(
        repo_root,
        grammar,
        source,
        &rules,
        GeneratedRuleEvaluationProfile::serial(),
        &datafox_environment,
    )
}

fn run_rules_on_file_sync_with_profile(
    repo_root: &Path,
    grammar: Grammar,
    source: &str,
    rules: &[PreparedCompiledRule],
    evaluation_profile: GeneratedRuleEvaluationProfile,
    datafox_environment: &DatafoxEnvironment,
) -> Result<Vec<LintViolation>> {
    let required_predicates = required_fact_predicates(rules);
    let (storage, locations) =
        load_or_build_facts(repo_root, grammar, source, &required_predicates)?;
    evaluate_rules(
        storage,
        locations,
        rules,
        evaluation_profile,
        datafox_environment,
    )
}

fn evaluate_rules(
    storage: InMemoryStorage,
    locations: HashMap<i64, NodeLocation>,
    rules: &[PreparedCompiledRule],
    evaluation_profile: GeneratedRuleEvaluationProfile,
    datafox_environment: &DatafoxEnvironment,
) -> Result<Vec<LintViolation>> {
    let mut violations = Vec::new();
    let mut seen = BTreeSet::new();
    let datafox = evaluation_profile.client_for(&storage, datafox_environment)?;

    for rule in rules {
        for query in &rule.prepared_queries {
            for substitution in datafox
                .eval_prepared(query)
                .with_context(|| format!("Failed to evaluate generated rule `{}`", rule.rule.id))?
            {
                let Some(Value::Integer(node_id)) = substitution.lookup(&rule.rule.primary) else {
                    continue;
                };
                let Some(location) = locations.get(node_id) else {
                    continue;
                };

                let message = render_message_template(&rule.rule.message_template, &substitution);
                let key = (
                    rule.rule.id.clone(),
                    location.line,
                    location.column,
                    message.clone(),
                );
                if seen.insert(key) {
                    violations.push(LintViolation {
                        line: location.line,
                        column: location.column,
                        message,
                        lint_name: rule.rule.name.clone(),
                        lint_id: rule.rule.id.clone(),
                    });
                }
            }
        }
    }

    violations.sort_by(|left, right| {
        (left.line, left.column, &left.lint_id, &left.message).cmp(&(
            right.line,
            right.column,
            &right.lint_id,
            &right.message,
        ))
    });
    Ok(violations)
}

fn rule_enabled(config: &LintbookConfig, rule: &CompiledRule) -> bool {
    config.is_lint_enabled(&rule.language, &rule.name)
}

fn render_message_template(template: &str, substitution: &Substitution) -> String {
    let mut message = template.to_string();
    for variable in substitution.variables() {
        let Some(value) = substitution.lookup(variable) else {
            continue;
        };
        let replacement = match value {
            Value::Integer(value) => value.to_string(),
            Value::String(value) => value.clone(),
        };
        message = message.replace(&format!("{{{variable}}}"), &replacement);
    }
    message
}

fn required_fact_predicates(rules: &[PreparedCompiledRule]) -> BTreeSet<String> {
    let mut predicates = BTreeSet::new();
    for rule in rules {
        for query in &rule.rule.queries {
            for clause in query.clauses() {
                match clause {
                    Clause::Atom(atom) | Clause::Negated(atom) => {
                        predicates.insert(atom.predicate);
                    }
                    Clause::Builtin { .. } => {}
                }
            }
        }
    }
    predicates
}

#[cfg(test)]
fn all_fact_predicates() -> BTreeSet<String> {
    [
        "node",
        "span",
        "location",
        "text",
        "trimmedText",
        "lowerText",
        "collapsedText",
        "literal",
        "intLiteralValue",
        "child",
        "argument",
        "statementExpression",
        "assignment",
        "comparison",
        "rangeBounds",
        "unitLike",
        "extremeValue",
        "moduleDecl",
        "attributeName",
        "attributeOf",
        "mistypedLiteralSuffix",
        "possibleMissingComma",
        "invisibleCharacter",
        "parent",
        "field",
        "named",
        "descendant",
        "nextSibling",
        "previousSibling",
        "nextCodeSibling",
        "sibling",
        "lineGap",
        "line",
        "nextLine",
        "previousLine",
        "pythonOutsideLoop",
        "pythonOutsideFunction",
        "pythonInsideFinally",
        "pythonOutsideExcept",
        "pythonScopeDeclaration",
        "pythonNameUse",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn predicate_fingerprint(predicates: &BTreeSet<String>) -> String {
    let joined = predicates.iter().cloned().collect::<Vec<_>>().join(",");
    sha256_hex(&joined)[..16].to_string()
}

fn load_or_build_facts(
    repo_root: &Path,
    grammar: Grammar,
    source: &str,
    required_predicates: &BTreeSet<String>,
) -> Result<(InMemoryStorage, HashMap<i64, NodeLocation>)> {
    let source_sha256 = sha256_hex(source);
    let language = grammar.name();
    let predicate_fingerprint = predicate_fingerprint(required_predicates);
    let cache_path = fact_cache_path(repo_root, language, &source_sha256, &predicate_fingerprint);

    if let Some(cached) = read_cached_facts(
        &cache_path,
        language,
        &source_sha256,
        &predicate_fingerprint,
    ) {
        return Ok(cached.into_runtime());
    }

    let facts = build_fact_set_for_predicates(
        grammar,
        source,
        &source_sha256,
        required_predicates,
        &predicate_fingerprint,
    )?;
    let _ = write_cached_facts(&cache_path, &facts);
    Ok(facts.into_runtime())
}

fn read_cached_facts(
    path: &Path,
    language: &str,
    source_sha256: &str,
    predicate_fingerprint: &str,
) -> Option<CachedFactSet> {
    let cached: CachedFactSet = bincode_decode(&fs::read(path).ok()?).ok()?;
    if cached.schema_version == FACT_SCHEMA_VERSION
        && cached.language == language
        && cached.source_sha256 == source_sha256
        && cached.predicate_fingerprint == predicate_fingerprint
    {
        Some(cached)
    } else {
        None
    }
}

fn write_cached_facts(path: &Path, facts: &CachedFactSet) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = bincode_encode(facts)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn fact_cache_path(
    repo_root: &Path,
    language: &str,
    source_sha256: &str,
    predicate_fingerprint: &str,
) -> PathBuf {
    cache_dir(repo_root)
        .join(FACTS_DIR)
        .join(format!("bincode-v{BINCODE_CACHE_FORMAT_VERSION}"))
        .join(language)
        .join(format!("{source_sha256}-{predicate_fingerprint}.bin"))
}

#[cfg(test)]
fn build_fact_set(grammar: Grammar, source: &str, source_sha256: &str) -> Result<CachedFactSet> {
    let required_predicates = all_fact_predicates();
    let predicate_fingerprint = predicate_fingerprint(&required_predicates);
    build_fact_set_for_predicates(
        grammar,
        source,
        source_sha256,
        &required_predicates,
        &predicate_fingerprint,
    )
}

fn build_fact_set_for_predicates(
    grammar: Grammar,
    source: &str,
    source_sha256: &str,
    required_predicates: &BTreeSet<String>,
    predicate_fingerprint: &str,
) -> Result<CachedFactSet> {
    let mut parser = Parser::new();
    let language = grammar.to_tree_sitter_language()?;
    parser.set_language(&language)?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("Failed to parse {} source", grammar.name()))?;

    let mut builder = FactBuilder {
        source,
        required_predicates,
        facts: BTreeMap::new(),
        locations: BTreeMap::new(),
        node_kinds: BTreeMap::new(),
        next_id: 1,
        next_line_id: -1,
    };
    builder.visit(tree.root_node(), None, 0, &[]);
    if builder.wants_any(&["line", "nextLine", "previousLine"]) {
        builder.insert_line_facts();
    }
    if builder.wants("invisibleCharacter") {
        builder.insert_invisible_character_facts();
    }
    let storage = InMemoryStorage::from_facts(builder.facts);
    Ok(CachedFactSet {
        schema_version: FACT_SCHEMA_VERSION,
        language: grammar.name().to_string(),
        source_sha256: source_sha256.to_string(),
        predicate_fingerprint: predicate_fingerprint.to_string(),
        storage,
        locations: builder.locations,
    })
}

struct FactBuilder<'a> {
    source: &'a str,
    required_predicates: &'a BTreeSet<String>,
    facts: BTreeMap<String, Vec<Vec<Value>>>,
    locations: BTreeMap<i64, NodeLocation>,
    node_kinds: BTreeMap<i64, String>,
    next_id: i64,
    next_line_id: i64,
}

#[derive(Debug)]
struct ChildFactInfo {
    id: i64,
    kind: String,
    text: String,
    is_named: bool,
    start_line: usize,
    end_line: usize,
}

impl<'a> FactBuilder<'a> {
    fn visit(
        &mut self,
        node: Node<'a>,
        parent: Option<i64>,
        child_index: usize,
        ancestors: &[i64],
    ) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        self.node_kinds.insert(id, node.kind().to_string());

        let start = node.start_position();
        let end = node.end_position();
        self.locations.insert(
            id,
            NodeLocation {
                line: start.row + 1,
                column: start.column + 1,
            },
        );

        if self.wants("node") {
            self.insert(
                "node",
                vec![
                    Value::integer(id),
                    Value::string(node.kind()),
                    Value::integer((start.row + 1) as i64),
                    Value::integer((start.column + 1) as i64),
                    Value::integer((end.row + 1) as i64),
                    Value::integer((end.column + 1) as i64),
                ],
            );
        }
        if self.wants("span") {
            self.insert(
                "span",
                vec![
                    Value::integer(id),
                    Value::integer(node.start_byte() as i64),
                    Value::integer(node.end_byte() as i64),
                ],
            );
        }
        if self.wants("location") {
            self.insert(
                "location",
                vec![
                    Value::integer(id),
                    Value::integer((start.row + 1) as i64),
                    Value::integer((start.column + 1) as i64),
                ],
            );
        }

        let node_text = node.utf8_text(self.source.as_bytes()).ok();
        if self.wants_any(&[
            "text",
            "trimmedText",
            "lowerText",
            "collapsedText",
            "literal",
            "intLiteralValue",
            "unitLike",
            "extremeValue",
            "attributeName",
            "mistypedLiteralSuffix",
        ]) {
            if let Some(text) = node_text {
                self.insert_text_facts(id, node.kind(), text);
            }
        }

        if self.wants("named") && node.is_named() {
            self.insert("named", vec![Value::integer(id)]);
        }

        if self.wants_any(&[
            "pythonOutsideLoop",
            "pythonOutsideFunction",
            "pythonInsideFinally",
            "pythonOutsideExcept",
            "pythonScopeDeclaration",
            "pythonNameUse",
        ]) {
            self.insert_python_context_facts(id, node, ancestors);
        }

        if let Some(parent_id) = parent {
            if self.wants("child") {
                self.insert(
                    "child",
                    vec![
                        Value::integer(parent_id),
                        Value::integer(id),
                        Value::integer(child_index as i64),
                    ],
                );
            }
            if self.wants("parent") {
                self.insert(
                    "parent",
                    vec![Value::integer(id), Value::integer(parent_id)],
                );
            }
        }

        if self.wants("descendant") {
            for ancestor in ancestors {
                self.insert(
                    "descendant",
                    vec![Value::integer(*ancestor), Value::integer(id)],
                );
            }
        }

        let mut next_ancestors = ancestors.to_vec();
        next_ancestors.push(id);

        let mut code_child_ids = Vec::new();
        let mut child_infos = Vec::new();
        let mut field_children = Vec::new();
        let mut argument_index = 0usize;
        for index in 0..node.child_count() {
            let child_index = index as u32;
            if let Some(child) = node.child(child_index) {
                let child_is_named = child.is_named();
                let child_kind = child.kind().to_string();
                let child_text = child
                    .utf8_text(self.source.as_bytes())
                    .unwrap_or_default()
                    .to_string();
                let child_start = child.start_position();
                let child_end = child.end_position();
                let child_id = self.visit(child, Some(id), index as usize, &next_ancestors);
                if child_is_named && !is_comment_kind(&child_kind) {
                    code_child_ids.push(child_id);
                }
                child_infos.push(ChildFactInfo {
                    id: child_id,
                    kind: child_kind,
                    text: child_text,
                    is_named: child_is_named,
                    start_line: child_start.row + 1,
                    end_line: if child_end.column == 0 && child_end.row > child_start.row {
                        child_end.row
                    } else {
                        child_end.row + 1
                    },
                });
                if self.wants("argument") && node.kind() == "arguments" && child_is_named {
                    self.insert(
                        "argument",
                        vec![
                            Value::integer(id),
                            Value::integer(child_id),
                            Value::integer(argument_index as i64),
                        ],
                    );
                    argument_index += 1;
                }
                if self.wants("statementExpression")
                    && node.kind() == "expression_statement"
                    && child_is_named
                {
                    self.insert(
                        "statementExpression",
                        vec![Value::integer(id), Value::integer(child_id)],
                    );
                }
                if let Some(field_name) = node.field_name_for_child(child_index) {
                    field_children.push((field_name.to_string(), child_id));
                    if self.wants("field") {
                        self.insert(
                            "field",
                            vec![
                                Value::integer(id),
                                Value::string(field_name),
                                Value::integer(child_id),
                            ],
                        );
                    }
                }
            }
        }
        self.insert_derived_node_facts(
            id,
            node.kind(),
            node_text.unwrap_or_default(),
            &field_children,
            &child_infos,
        );

        if self.wants_any(&["nextSibling", "previousSibling", "sibling", "lineGap"]) {
            for adjacent in child_infos.windows(2) {
                let left_info = &adjacent[0];
                let right_info = &adjacent[1];
                let left = left_info.id;
                let right = right_info.id;
                if self.wants("nextSibling") {
                    self.insert(
                        "nextSibling",
                        vec![Value::integer(left), Value::integer(right)],
                    );
                }
                if self.wants("previousSibling") {
                    self.insert(
                        "previousSibling",
                        vec![Value::integer(right), Value::integer(left)],
                    );
                }
                if self.wants("sibling") {
                    self.insert(
                        "sibling",
                        vec![
                            Value::integer(id),
                            Value::integer(left),
                            Value::integer(right),
                        ],
                    );
                }
                if self.wants("lineGap") {
                    let gap = right_info
                        .start_line
                        .saturating_sub(left_info.end_line)
                        .saturating_sub(1);
                    self.insert(
                        "lineGap",
                        vec![
                            Value::integer(left),
                            Value::integer(right),
                            Value::integer(gap as i64),
                        ],
                    );
                }
            }
        }
        if self.wants("nextCodeSibling") {
            for adjacent in code_child_ids.windows(2) {
                self.insert(
                    "nextCodeSibling",
                    vec![Value::integer(adjacent[0]), Value::integer(adjacent[1])],
                );
            }
        }

        id
    }

    fn insert_invisible_character_facts(&mut self) {
        for (line_index, line) in self.source.lines().enumerate() {
            for (byte_column, ch) in line.char_indices() {
                if let Some(name) = invisible_character_name(ch) {
                    let id = self.next_line_id;
                    self.next_line_id -= 1;
                    self.locations.insert(
                        id,
                        NodeLocation {
                            line: line_index + 1,
                            column: byte_column + 1,
                        },
                    );
                    self.insert(
                        "invisibleCharacter",
                        vec![
                            Value::integer(id),
                            Value::string(name),
                            Value::string(format!("U+{:04X}", ch as u32)),
                        ],
                    );
                }
            }
        }
    }

    fn insert_python_context_facts(&mut self, id: i64, node: Node<'a>, ancestors: &[i64]) {
        if self.wants("pythonOutsideLoop") && !self.has_python_loop_context(ancestors) {
            self.insert("pythonOutsideLoop", vec![Value::integer(id)]);
        }
        if self.wants("pythonOutsideFunction") && !self.has_python_function_context(ancestors) {
            self.insert("pythonOutsideFunction", vec![Value::integer(id)]);
        }
        if self.wants("pythonInsideFinally") && self.has_python_finally_context(ancestors) {
            self.insert("pythonInsideFinally", vec![Value::integer(id)]);
        }
        if self.wants("pythonOutsideExcept") && !self.has_python_except_context(ancestors) {
            self.insert("pythonOutsideExcept", vec![Value::integer(id)]);
        }
        if self.wants("pythonScopeDeclaration")
            && matches!(node.kind(), "global_statement" | "nonlocal_statement")
        {
            self.insert_python_scope_declarations(id, node, ancestors);
        }
        if self.wants("pythonNameUse")
            && node.kind() == "identifier"
            && !self.has_python_import_context(ancestors)
        {
            if let Ok(name) = node.utf8_text(self.source.as_bytes()) {
                self.insert("pythonNameUse", vec![Value::string(name)]);
            }
        }
    }

    fn insert_python_scope_declarations(&mut self, id: i64, node: Node<'a>, ancestors: &[i64]) {
        let declaration_kind = match node.kind() {
            "global_statement" => "global",
            "nonlocal_statement" => "nonlocal",
            _ => return,
        };
        let Some(scope) = self.nearest_python_function(ancestors) else {
            return;
        };

        for index in 0..node.child_count() {
            let Some(child) = node.child(index as u32) else {
                continue;
            };
            if child.kind() != "identifier" {
                continue;
            }
            if let Ok(name) = child.utf8_text(self.source.as_bytes()) {
                self.insert(
                    "pythonScopeDeclaration",
                    vec![
                        Value::integer(scope),
                        Value::string(declaration_kind),
                        Value::string(name),
                        Value::integer(id),
                    ],
                );
            }
        }
    }

    fn has_python_loop_context(&self, ancestors: &[i64]) -> bool {
        for kind in self.ancestor_kinds_rev(ancestors) {
            if is_python_scope_boundary(kind) {
                return false;
            }
            if matches!(kind, "for_statement" | "while_statement") {
                return true;
            }
        }
        false
    }

    fn has_python_function_context(&self, ancestors: &[i64]) -> bool {
        self.ancestor_kinds_rev(ancestors).any(|kind| {
            matches!(
                kind,
                "function_definition"
                    | "async_function_definition"
                    | "lambda"
                    | "generator_expression"
            )
        })
    }

    fn has_python_finally_context(&self, ancestors: &[i64]) -> bool {
        for kind in self.ancestor_kinds_rev(ancestors) {
            if is_python_scope_boundary(kind) {
                return false;
            }
            if kind == "finally_clause" {
                return true;
            }
        }
        false
    }

    fn has_python_except_context(&self, ancestors: &[i64]) -> bool {
        for kind in self.ancestor_kinds_rev(ancestors) {
            if kind == "except_clause" {
                return true;
            }
            if matches!(
                kind,
                "try_statement"
                    | "function_definition"
                    | "async_function_definition"
                    | "class_definition"
                    | "module"
            ) {
                return false;
            }
        }
        false
    }

    fn has_python_import_context(&self, ancestors: &[i64]) -> bool {
        self.ancestor_kinds_rev(ancestors)
            .any(|kind| matches!(kind, "import_statement" | "import_from_statement"))
    }

    fn nearest_python_function(&self, ancestors: &[i64]) -> Option<i64> {
        ancestors.iter().rev().find_map(|ancestor| {
            self.node_kinds
                .get(ancestor)
                .is_some_and(|kind| {
                    matches!(
                        kind.as_str(),
                        "function_definition" | "async_function_definition"
                    )
                })
                .then_some(*ancestor)
        })
    }

    fn ancestor_kinds_rev<'b>(
        &'b self,
        ancestors: &'b [i64],
    ) -> impl Iterator<Item = &'b str> + 'b {
        ancestors
            .iter()
            .rev()
            .filter_map(|ancestor| self.node_kinds.get(ancestor).map(String::as_str))
    }

    fn insert_derived_node_facts(
        &mut self,
        id: i64,
        kind: &str,
        text: &str,
        field_children: &[(String, i64)],
        child_infos: &[ChildFactInfo],
    ) {
        if self.wants("assignment") && kind == "assignment_expression" {
            if let (Some(left), Some(right)) = (
                field_child(field_children, "left"),
                field_child(field_children, "right"),
            ) {
                self.insert(
                    "assignment",
                    vec![
                        Value::integer(id),
                        Value::integer(left),
                        Value::integer(right),
                    ],
                );
            }
        }

        if self.wants("comparison") && kind == "binary_expression" {
            if let (Some(left), Some(operator), Some(right)) = (
                field_child(field_children, "left"),
                field_child(field_children, "operator"),
                field_child(field_children, "right"),
            ) {
                if child_text(child_infos, operator).is_some_and(is_comparison_operator) {
                    self.insert(
                        "comparison",
                        vec![
                            Value::integer(id),
                            Value::integer(left),
                            Value::integer(operator),
                            Value::integer(right),
                        ],
                    );
                }
            }
        }

        if self.wants("rangeBounds")
            && matches!(kind, "range_expression" | "range_inclusive_expression")
        {
            let left = field_child(field_children, "left").or_else(|| {
                child_infos
                    .iter()
                    .find_map(|child| child.is_named.then_some(child.id))
            });
            let right = field_child(field_children, "right").or_else(|| {
                child_infos
                    .iter()
                    .rev()
                    .find_map(|child| child.is_named.then_some(child.id))
            });
            if let (Some(left), Some(right)) = (left, right) {
                if left == right {
                    return;
                }
                self.insert(
                    "rangeBounds",
                    vec![
                        Value::integer(id),
                        Value::integer(left),
                        Value::integer(right),
                    ],
                );
            }
        }

        if self.wants("moduleDecl") && kind == "mod_item" {
            if let Some(name) = field_child(field_children, "name")
                .and_then(|name| child_text(child_infos, name).map(str::to_string))
            {
                self.insert("moduleDecl", vec![Value::integer(id), Value::string(name)]);
            }
        }

        if self.wants("attributeName") && kind == "attribute_item" {
            if let Some(name) = extract_attribute_name(text) {
                self.insert(
                    "attributeName",
                    vec![Value::integer(id), Value::string(name)],
                );
            }
        }

        if self.wants("attributeOf") {
            self.insert_attribute_of_facts(id, kind, child_infos);
        }

        if self.wants("possibleMissingComma") && kind == "array_expression" {
            self.insert_possible_missing_comma_facts(child_infos);
        }
    }

    fn insert_attribute_of_facts(
        &mut self,
        target_id: i64,
        target_kind: &str,
        child_infos: &[ChildFactInfo],
    ) {
        if is_attribute_target_kind(target_kind) {
            for child in child_infos {
                if child.kind == "attribute_item" {
                    if let Some(name) = extract_attribute_name(&child.text) {
                        self.insert(
                            "attributeOf",
                            vec![
                                Value::integer(target_id),
                                Value::integer(child.id),
                                Value::string(name),
                            ],
                        );
                    }
                }
            }
        }

        let mut pending = Vec::new();
        for child in child_infos {
            if child.kind == "attribute_item" {
                if let Some(name) = extract_attribute_name(&child.text) {
                    pending.push((child.id, name));
                }
                continue;
            }

            if is_comment_kind(&child.kind) {
                continue;
            }

            if child.is_named && !pending.is_empty() {
                for (attribute_id, name) in pending.drain(..) {
                    self.insert(
                        "attributeOf",
                        vec![
                            Value::integer(child.id),
                            Value::integer(attribute_id),
                            Value::string(name),
                        ],
                    );
                }
            } else if child.is_named {
                pending.clear();
            }
        }
    }

    fn insert_possible_missing_comma_facts(&mut self, child_infos: &[ChildFactInfo]) {
        let elements = child_infos
            .iter()
            .enumerate()
            .filter(|(_, child)| is_array_element_kind(&child.kind))
            .collect::<Vec<_>>();

        for adjacent in elements.windows(2) {
            let (previous_index, previous) = adjacent[0];
            let (current_index, current) = adjacent[1];
            let has_comma_between = child_infos[previous_index + 1..current_index]
                .iter()
                .any(|child| child.kind == ",");
            if has_comma_between {
                continue;
            }
            if current.start_line > previous.end_line && looks_like_missing_comma(previous, current)
            {
                self.insert("possibleMissingComma", vec![Value::integer(current.id)]);
            }
        }
    }

    fn insert_line_facts(&mut self) {
        if self.source.is_empty() {
            self.insert_line_fact(1, "", 0);
            return;
        }

        let mut start_byte = 0usize;
        let mut previous_line_id = None;
        let parts = self.source.split('\n').collect::<Vec<_>>();
        for (index, raw_line) in parts.iter().enumerate() {
            if index == parts.len() - 1 && raw_line.is_empty() && self.source.ends_with('\n') {
                break;
            }
            let line_text = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            let line_id = self.insert_line_fact(index + 1, line_text, start_byte);
            if let Some(previous_line_id) = previous_line_id {
                if self.wants("nextLine") {
                    self.insert(
                        "nextLine",
                        vec![Value::integer(previous_line_id), Value::integer(line_id)],
                    );
                }
                if self.wants("previousLine") {
                    self.insert(
                        "previousLine",
                        vec![Value::integer(line_id), Value::integer(previous_line_id)],
                    );
                }
            }
            previous_line_id = Some(line_id);
            start_byte += raw_line.len() + 1;
        }
    }

    fn insert_line_fact(&mut self, line_number: usize, text: &str, start_byte: usize) -> i64 {
        let id = self.next_line_id;
        self.next_line_id -= 1;
        let end_byte = start_byte + text.len();
        self.locations.insert(
            id,
            NodeLocation {
                line: line_number,
                column: 1,
            },
        );
        if self.wants("line") {
            self.insert(
                "line",
                vec![
                    Value::integer(id),
                    Value::integer(line_number as i64),
                    Value::string(text),
                    Value::integer(start_byte as i64),
                    Value::integer(end_byte as i64),
                ],
            );
        }
        if self.wants("location") {
            self.insert(
                "location",
                vec![
                    Value::integer(id),
                    Value::integer(line_number as i64),
                    Value::integer(1),
                ],
            );
        }
        id
    }

    fn insert_text_facts(&mut self, id: i64, kind: &str, text: &str) {
        if self.wants("text") {
            self.insert("text", vec![Value::integer(id), Value::string(text)]);
        }
        if self.wants("trimmedText") {
            self.insert(
                "trimmedText",
                vec![Value::integer(id), Value::string(text.trim())],
            );
        }
        if self.wants("lowerText") {
            self.insert(
                "lowerText",
                vec![Value::integer(id), Value::string(text.to_ascii_lowercase())],
            );
        }
        if self.wants("collapsedText") {
            self.insert(
                "collapsedText",
                vec![Value::integer(id), Value::string(collapse_whitespace(text))],
            );
        }

        if self.wants("literal") {
            if let Some((literal_kind, normalized)) = normalize_literal_text(kind, text) {
                self.insert(
                    "literal",
                    vec![
                        Value::integer(id),
                        Value::string(literal_kind),
                        Value::string(text),
                        Value::string(normalized),
                    ],
                );
            }
        }
        if self.wants("intLiteralValue") && kind == "integer_literal" {
            if let Some(value) = parse_integer_literal_value(text) {
                self.insert(
                    "intLiteralValue",
                    vec![Value::integer(id), Value::integer(value)],
                );
            }
        }
        if self.wants("unitLike") && is_unit_like_text(text) {
            self.insert("unitLike", vec![Value::integer(id)]);
        }
        if self.wants("extremeValue") && is_extreme_value_text(text) {
            self.insert("extremeValue", vec![Value::integer(id)]);
        }
        if self.wants("mistypedLiteralSuffix")
            && kind == "integer_literal"
            && detect_mistyped_literal_suffix(text)
        {
            self.insert("mistypedLiteralSuffix", vec![Value::integer(id)]);
        }
    }

    fn insert(&mut self, predicate: impl Into<String>, tuple: Vec<Value>) {
        self.facts.entry(predicate.into()).or_default().push(tuple);
    }

    fn wants(&self, predicate: &str) -> bool {
        self.required_predicates.contains(predicate)
    }

    fn wants_any(&self, predicates: &[&str]) -> bool {
        predicates.iter().any(|predicate| self.wants(predicate))
    }
}

fn field_child(field_children: &[(String, i64)], field_name: &str) -> Option<i64> {
    field_children
        .iter()
        .find_map(|(name, id)| (name == field_name).then_some(*id))
}

fn child_text(child_infos: &[ChildFactInfo], id: i64) -> Option<&str> {
    child_infos
        .iter()
        .find_map(|child| (child.id == id).then_some(child.text.as_str()))
}

fn is_comment_kind(kind: &str) -> bool {
    matches!(kind, "line_comment" | "block_comment")
}

fn is_python_scope_boundary(kind: &str) -> bool {
    matches!(
        kind,
        "function_definition" | "async_function_definition" | "class_definition" | "lambda"
    )
}

fn is_comparison_operator(operator: &str) -> bool {
    matches!(operator, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_integer_literal_value(text: &str) -> Option<i64> {
    let mut normalized = text.trim().replace('_', "").to_ascii_lowercase();
    for suffix in INTEGER_SUFFIXES {
        if normalized.ends_with(suffix) {
            normalized.truncate(normalized.len() - suffix.len());
            break;
        }
    }

    if normalized.is_empty() {
        return None;
    }

    if let Some(hex) = normalized.strip_prefix("0x") {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(octal) = normalized.strip_prefix("0o") {
        i64::from_str_radix(octal, 8).ok()
    } else if let Some(binary) = normalized.strip_prefix("0b") {
        i64::from_str_radix(binary, 2).ok()
    } else {
        normalized.parse::<i64>().ok()
    }
}

fn is_unit_like_text(text: &str) -> bool {
    let text = text.trim();
    text == "()"
        || matches!(
            text,
            "println!()" | "print!()" | "panic!()" | "unreachable!()" | "todo!()"
        )
}

fn is_extreme_value_text(text: &str) -> bool {
    let text = text.trim();
    matches!(
        text,
        "u8::MAX"
            | "u8::MIN"
            | "u16::MAX"
            | "u16::MIN"
            | "u32::MAX"
            | "u32::MIN"
            | "u64::MAX"
            | "u64::MIN"
            | "u128::MAX"
            | "u128::MIN"
            | "usize::MAX"
            | "usize::MIN"
            | "i8::MAX"
            | "i8::MIN"
            | "i16::MAX"
            | "i16::MIN"
            | "i32::MAX"
            | "i32::MIN"
            | "i64::MAX"
            | "i64::MIN"
            | "i128::MAX"
            | "i128::MIN"
            | "isize::MAX"
            | "isize::MIN"
            | "f32::INFINITY"
            | "f32::NEG_INFINITY"
            | "f64::INFINITY"
            | "f64::NEG_INFINITY"
    ) || parse_integer_literal_value(text)
        .is_some_and(|value| matches!(value, 255 | 65_535 | 4_294_967_295))
}

fn detect_mistyped_literal_suffix(literal: &str) -> bool {
    let cleaned = literal.replace('_', "");

    if let Some(underscore_pos) = literal.rfind('_') {
        let suffix = &literal[underscore_pos + 1..];
        return matches!(suffix, "8" | "16" | "32" | "64" | "128" | "size");
    }

    if cleaned.len() <= 2 {
        return false;
    }

    if cleaned.ends_with("32")
        && !cleaned.ends_with("f32")
        && !cleaned.ends_with("i32")
        && !cleaned.ends_with("u32")
    {
        return literal[..literal.len() - 2]
            .chars()
            .all(|ch| ch.is_ascii_digit());
    }
    if cleaned.ends_with("64")
        && !cleaned.ends_with("f64")
        && !cleaned.ends_with("i64")
        && !cleaned.ends_with("u64")
    {
        return literal[..literal.len() - 2]
            .chars()
            .all(|ch| ch.is_ascii_digit());
    }
    if cleaned.ends_with("16") && !cleaned.ends_with("i16") && !cleaned.ends_with("u16") {
        return literal[..literal.len() - 2]
            .chars()
            .all(|ch| ch.is_ascii_digit());
    }
    if cleaned.ends_with('8') && !cleaned.ends_with("i8") && !cleaned.ends_with("u8") {
        let base = &literal[..literal.len() - 1];
        return base.chars().all(|ch| ch.is_ascii_digit())
            && base.parse::<u32>().is_ok_and(|value| value <= 255);
    }

    false
}

fn extract_attribute_name(attribute_text: &str) -> Option<String> {
    let content = attribute_text
        .trim()
        .strip_prefix("#[")
        .and_then(|text| text.strip_suffix(']'))?;
    content
        .split_whitespace()
        .next()
        .map(|first| first.split('(').next().unwrap_or(first).to_string())
        .filter(|name| !name.is_empty())
}

fn is_attribute_target_kind(kind: &str) -> bool {
    matches!(
        kind,
        "function_item"
            | "struct_item"
            | "enum_item"
            | "impl_item"
            | "trait_item"
            | "mod_item"
            | "const_item"
            | "static_item"
            | "type_item"
            | "field_declaration"
            | "parameter"
    )
}

fn is_array_element_kind(kind: &str) -> bool {
    !matches!(kind, "[" | "]" | "," | ";" | "(" | ")")
}

fn looks_like_missing_comma(previous: &ChildFactInfo, current: &ChildFactInfo) -> bool {
    (ends_with_literal_or_identifier(previous) && starts_with_unary_operator(current))
        || (is_simple_literal_or_identifier(previous) && is_simple_literal_or_identifier(current))
        || (is_complex_expression(previous) && is_simple_literal_or_identifier(current))
}

fn ends_with_literal_or_identifier(node: &ChildFactInfo) -> bool {
    matches!(
        node.kind.as_str(),
        "integer_literal" | "float_literal" | "string_literal" | "identifier" | "unary_expression"
    )
}

fn starts_with_unary_operator(node: &ChildFactInfo) -> bool {
    node.kind == "unary_expression"
        && node
            .text
            .trim_start()
            .chars()
            .next()
            .is_some_and(|ch| matches!(ch, '-' | '+' | '!'))
}

fn is_simple_literal_or_identifier(node: &ChildFactInfo) -> bool {
    matches!(
        node.kind.as_str(),
        "integer_literal" | "float_literal" | "string_literal" | "identifier" | "boolean_literal"
    ) || (node.kind == "ERROR"
        && node
            .text
            .trim()
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
}

fn is_complex_expression(node: &ChildFactInfo) -> bool {
    matches!(
        node.kind.as_str(),
        "binary_expression"
            | "call_expression"
            | "field_expression"
            | "method_call_expression"
            | "macro_invocation"
    )
}

fn invisible_character_name(ch: char) -> Option<&'static str> {
    let name = match ch {
        '\u{200B}' => "Zero Width Space",
        '\u{200C}' => "Zero Width Non-Joiner",
        '\u{200D}' => "Zero Width Joiner",
        '\u{FEFF}' => "Zero Width No-Break Space (BOM)",
        '\u{2060}' => "Word Joiner",
        '\u{202A}' => "Left-to-Right Embedding",
        '\u{202B}' => "Right-to-Left Embedding",
        '\u{202C}' => "Pop Directional Formatting",
        '\u{202D}' => "Left-to-Right Override",
        '\u{202E}' => "Right-to-Left Override",
        '\u{2066}' => "Left-to-Right Isolate",
        '\u{2067}' => "Right-to-Left Isolate",
        '\u{2068}' => "First Strong Isolate",
        '\u{2069}' => "Pop Directional Isolate",
        '\u{00AD}' => "Soft Hyphen",
        '\u{034F}' => "Combining Grapheme Joiner",
        '\u{061C}' => "Arabic Letter Mark",
        '\u{115F}' => "Hangul Choseong Filler",
        '\u{1160}' => "Hangul Jungseong Filler",
        '\u{17B4}' => "Khmer Vowel Inherent AQ",
        '\u{17B5}' => "Khmer Vowel Inherent AA",
        '\u{180E}' => "Mongolian Vowel Separator",
        '\u{3164}' => "Hangul Filler",
        '\u{FFA0}' => "Halfwidth Hangul Filler",
        '\u{FE00}'..='\u{FE0F}' | '\u{E0100}'..='\u{E01EF}' => "Variation Selector",
        _ if is_invisible_by_category(ch) => "Unknown Invisible Character",
        _ => return None,
    };
    Some(name)
}

fn is_invisible_by_category(ch: char) -> bool {
    let code = ch as u32;
    if ((0x00..=0x1F).contains(&code) || (0x7F..=0x9F).contains(&code))
        && !matches!(ch, '\t' | '\n' | '\r')
    {
        return true;
    }

    matches!(
        code,
        0x00AD | 0x061C | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2069 | 0xFEFF | 0xFFF9..=0xFFFB
    )
}

fn normalize_literal_text(kind: &str, text: &str) -> Option<(&'static str, String)> {
    match kind {
        "integer_literal" => Some(("integer", normalize_numeric_literal(text, INTEGER_SUFFIXES))),
        "float_literal" => Some(("float", normalize_numeric_literal(text, FLOAT_SUFFIXES))),
        "string_literal" | "raw_string_literal" => Some(("string", normalize_string_literal(text))),
        "char_literal" => Some(("char", normalize_char_literal(text))),
        "boolean_literal" => Some(("boolean", text.trim().to_ascii_lowercase())),
        "unit_expression" => Some(("unit", "()".to_string())),
        _ if text.trim() == "()" => Some(("unit", "()".to_string())),
        _ => None,
    }
}

const INTEGER_SUFFIXES: &[&str] = &[
    "usize", "isize", "u128", "i128", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
];
const FLOAT_SUFFIXES: &[&str] = &["f32", "f64"];

fn normalize_numeric_literal(text: &str, suffixes: &[&str]) -> String {
    let mut normalized = text.trim().replace('_', "").to_ascii_lowercase();
    for suffix in suffixes {
        if normalized.ends_with(suffix) {
            normalized.truncate(normalized.len() - suffix.len());
            break;
        }
    }
    normalized
}

fn normalize_string_literal(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(raw_body) = trimmed.strip_prefix('r') {
        let hashes = raw_body.chars().take_while(|ch| *ch == '#').count();
        let quote_start = 1 + hashes;
        let quote_end = trimmed.len().saturating_sub(1 + hashes);
        if trimmed.as_bytes().get(quote_start) == Some(&b'"') && quote_end > quote_start {
            return trimmed[quote_start + 1..quote_end].to_string();
        }
    }
    trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

fn normalize_char_literal(text: &str) -> String {
    let trimmed = text.trim();
    trimmed
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(trimmed)
        .to_string()
}

fn sha256_hex(source: &str) -> String {
    sha256_hex_bytes(source.as_bytes())
}

fn sha256_hex_bytes(source: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source);
    hex::encode(hasher.finalize())
}

pub fn sanitize_file_stem(input: &str) -> String {
    let mut output = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    output.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn compiles_active_rust_rule() {
        let temp = tempdir().unwrap();
        let rules = rules_dir(temp.path());
        fs::create_dir_all(&rules).unwrap();
        let gen = gen_dir(temp.path());
        fs::create_dir_all(&gen).unwrap();
        fs::write(
            rules.join("no-dbg.md"),
            r#"---
id: rust.no-dbg
lang: rust
---

Avoid dbg! in committed code.
"#,
        )
        .unwrap();
        fs::write(
            gen.join("no-dbg.df"),
            r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")"#,
        )
        .unwrap();

        let report = compile_project(temp.path()).unwrap();
        assert_eq!(report.compiled, vec!["rust.no-dbg"]);
        assert!(generated_query_path(temp.path(), &rules.join("no-dbg.md"))
            .unwrap()
            .exists());
        assert_eq!(
            fs::read_to_string(gen.join("no-dbg.df")).unwrap(),
            "node(Node, \"macro_invocation\", _, _, _, _),\n  text(Node, Text),\n  contains(Text, \"dbg!\")\n"
        );
        assert!(compiled_rule_path(temp.path(), "rust.no-dbg").exists());
    }

    #[test]
    fn old_compiled_artifacts_are_reported_as_stale() {
        let temp = tempdir().unwrap();
        let rules = rules_dir(temp.path());
        fs::create_dir_all(&rules).unwrap();
        let gen = gen_dir(temp.path());
        fs::create_dir_all(&gen).unwrap();
        fs::write(
            rules.join("no-dbg.md"),
            r#"---
id: rust.no-dbg
lang: rust
---

Avoid dbg! in committed code.
"#,
        )
        .unwrap();
        fs::write(
            gen.join("no-dbg.df"),
            r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")"#,
        )
        .unwrap();
        fs::write(
            compiled_rule_path(temp.path(), "rust.no-dbg"),
            r#"{"schema_version":1,"id":"rust.no-dbg"}"#,
        )
        .unwrap();

        let error = load_compiled_rules(temp.path()).unwrap_err();
        assert!(error.to_string().contains("Run `lintbook compile`"));
    }

    #[tokio::test]
    async fn generated_rule_flags_rust_source() {
        let temp = tempdir().unwrap();
        let rules = rules_dir(temp.path());
        fs::create_dir_all(&rules).unwrap();
        let gen = gen_dir(temp.path());
        fs::create_dir_all(&gen).unwrap();
        fs::write(
            rules.join("no-dbg.md"),
            r#"---
id: rust.no-dbg
lang: rust
---

Avoid dbg! in committed code.
"#,
        )
        .unwrap();
        fs::write(
            gen.join("no-dbg.df"),
            r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")"#,
        )
        .unwrap();
        compile_project(temp.path()).unwrap();

        let source_path = temp.path().join("main.rs");
        fs::write(&source_path, "fn main() { dbg!(1); }\n").unwrap();
        let results = vec![LintResult {
            file_path: source_path.clone(),
            duration: std::time::Duration::from_millis(0),
            status: LintStatus::Ok,
            violations: vec![],
            language: Some(Grammar::Rust),
        }];

        let config = LintbookConfig::new(vec!["rust"]);
        let generated = run_generated_rules(temp.path(), &config, &results)
            .await
            .unwrap();
        assert_eq!(generated[&source_path][0].lint_id, "rust.no-dbg");
    }

    #[test]
    fn generated_rule_runner_prepares_queries_once_by_grammar() {
        let temp = tempdir().unwrap();
        let rules = rules_dir(temp.path());
        fs::create_dir_all(&rules).unwrap();
        let gen = gen_dir(temp.path());
        fs::create_dir_all(&gen).unwrap();
        fs::write(
            rules.join("no-dbg.md"),
            r#"---
id: rust.no-dbg
lang: rust
---

Avoid dbg! in committed code.
"#,
        )
        .unwrap();
        fs::write(
            gen.join("no-dbg.df"),
            r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")"#,
        )
        .unwrap();
        compile_project(temp.path()).unwrap();

        let config = LintbookConfig::new(vec!["rust"]);
        let expected_queries = load_all_rules(temp.path(), &config)
            .unwrap()
            .iter()
            .map(|rule| rule.queries.len())
            .sum::<usize>();
        let runner = GeneratedRuleRunner::new(temp.path(), &config).unwrap();

        assert!(runner.rules_by_language.contains_key(&Grammar::Rust));
        assert_eq!(runner.prepared_query_count(), expected_queries);
    }

    #[test]
    fn prepared_queries_are_cached_on_disk() {
        let temp = tempdir().unwrap();
        let query = datafox::parse_query(
            r#"node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")"#,
        )
        .unwrap();
        let key = PreparedQueryKey::new(query.clone());
        let storage = FilePreparedQueryStorage::new(temp.path());
        let environment = DatafoxEnvironment::builder()
            .with_prepared_query_storage(storage.clone())
            .build();

        let prepared = environment.prepare(&query).unwrap();
        let path = storage.path_for(&key).unwrap();

        assert!(path.exists());

        let reloaded_storage = FilePreparedQueryStorage::new(temp.path());
        let reloaded_environment = DatafoxEnvironment::builder()
            .with_prepared_query_storage(reloaded_storage)
            .build();
        let reloaded = reloaded_environment.prepare(&query).unwrap();

        assert_eq!(prepared.as_ref(), reloaded.as_ref());
    }

    #[test]
    fn rust_facts_are_cached_by_source_hash() {
        let temp = tempdir().unwrap();
        let source = "fn main() { dbg!(1); }\n";
        let source_sha256 = sha256_hex(source);
        let required_predicates = all_fact_predicates();
        let predicate_fingerprint = predicate_fingerprint(&required_predicates);

        let (_storage, locations) =
            load_or_build_facts(temp.path(), Grammar::Rust, source, &required_predicates).unwrap();
        assert!(!locations.is_empty());

        let path = fact_cache_path(temp.path(), "rust", &source_sha256, &predicate_fingerprint);
        assert!(path.exists());

        let cached: CachedFactSet = bincode_decode(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(cached.schema_version, FACT_SCHEMA_VERSION);
        assert_eq!(cached.language, "rust");
        assert_eq!(cached.source_sha256, source_sha256);
        assert_eq!(cached.predicate_fingerprint, predicate_fingerprint);
        assert!(!cached
            .storage
            .facts_matching("node", &[None, None, None, None, None, None])
            .is_empty());
        assert!(!cached.locations.is_empty());

        let empty_cache = CachedFactSet {
            schema_version: FACT_SCHEMA_VERSION,
            language: "rust".to_string(),
            source_sha256: cached.source_sha256,
            predicate_fingerprint,
            storage: InMemoryStorage::new(),
            locations: BTreeMap::new(),
        };
        fs::write(&path, bincode_encode(&empty_cache).unwrap()).unwrap();

        let (_storage, locations) =
            load_or_build_facts(temp.path(), Grammar::Rust, source, &required_predicates).unwrap();
        assert!(locations.is_empty());
    }

    #[test]
    fn rust_fact_schema_includes_parent_sibling_and_line_order_facts() {
        let facts = build_fact_set(
            Grammar::Rust,
            "fn main() {\nlet UPPER = \"Text\";\nlet y = 2_u32;\nlet unit = ();\n}\n",
            "test-sha",
        )
        .unwrap();
        let predicates = facts.storage.predicates().collect::<HashSet<_>>();

        assert_eq!(facts.schema_version, FACT_SCHEMA_VERSION);
        assert!(predicates.contains("parent"));
        assert!(predicates.contains("nextSibling"));
        assert!(predicates.contains("previousSibling"));
        assert!(predicates.contains("sibling"));
        assert!(predicates.contains("nextLine"));
        assert!(predicates.contains("previousLine"));
        assert!(predicates.contains("trimmedText"));
        assert!(predicates.contains("lowerText"));
        assert!(predicates.contains("literal"));

        assert!(facts
            .storage
            .facts_matching("literal", &[None, None, None, None])
            .into_iter()
            .any(|tuple| {
                tuple.get(1) == Some(&Value::string("integer"))
                    && tuple.get(2) == Some(&Value::string("2_u32"))
                    && tuple.get(3) == Some(&Value::string("2"))
            }));
    }

    #[test]
    fn compiles_embedded_builtin_rules() {
        let rules = compile_builtin_rules().unwrap();
        assert_eq!(rules.len(), 96);
        assert!(rules.iter().any(|rule| {
            rule.id == "RS013"
                && rule.name == "eq-op"
                && rule.language == "rust"
                && !rule.queries.is_empty()
        }));
        assert!(rules.iter().any(|rule| {
            rule.id == "PY003"
                && rule.name == "no-os-getenv"
                && rule.language == "python"
                && !rule.queries.is_empty()
        }));

        let infos = builtin_rule_infos().unwrap();
        assert!(infos
            .iter()
            .any(|info| { info.id == "RS095" && info.name == "four-forward-slashes" }));
    }

    async fn assert_builtin_rule_for_grammar(
        grammar: Grammar,
        rule_id: &str,
        positives: &[&str],
        negatives: &[&str],
    ) {
        let temp = tempdir().unwrap();
        let rules = compile_builtin_rules()
            .unwrap()
            .into_iter()
            .filter(|rule| rule.id == rule_id)
            .collect::<Vec<_>>();
        assert_eq!(rules.len(), 1, "missing built-in rule {rule_id}");

        for positive in positives {
            let violations =
                run_rules_on_file_sync(temp.path(), grammar, positive, &rules).unwrap();
            assert!(
                violations
                    .iter()
                    .any(|violation| violation.lint_id == rule_id),
                "{rule_id} did not flag positive sample:\n{positive}"
            );
        }

        for negative in negatives {
            let violations =
                run_rules_on_file_sync(temp.path(), grammar, negative, &rules).unwrap();
            assert!(
                violations
                    .iter()
                    .all(|violation| violation.lint_id != rule_id),
                "{rule_id} flagged negative sample:\n{negative}\n{violations:#?}"
            );
        }
    }

    async fn assert_builtin_rule(rule_id: &str, positives: &[&str], negatives: &[&str]) {
        assert_builtin_rule_for_grammar(Grammar::Rust, rule_id, positives, negatives).await;
    }

    #[tokio::test]
    async fn embedded_builtin_rule_flags_rust_source() {
        let temp = tempdir().unwrap();
        let rules = compile_builtin_rules()
            .unwrap()
            .into_iter()
            .filter(|rule| rule.id == "RS013")
            .collect::<Vec<_>>();

        let violations = run_rules_on_file_sync(
            temp.path(),
            Grammar::Rust,
            "fn main() { let x = 1; let _ = x == x; }\n",
            &rules,
        )
        .unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "RS013");
        assert_eq!(violations[0].lint_name, "eq-op");
    }

    #[tokio::test]
    async fn embedded_eq_op_rule_skips_intentional_cases() {
        let temp = tempdir().unwrap();
        let rules = compile_builtin_rules()
            .unwrap()
            .into_iter()
            .filter(|rule| rule.id == "RS013")
            .collect::<Vec<_>>();

        let source = r#"
fn main() {
    let x = 1;
    let _ = x + x;
    let _ = x * x;
    const MAX_SIZE: usize = 1024;
    let _ = MAX_SIZE & MAX_SIZE;
    let float_val = 3.14f64;
    if float_val != float_val {}
}
"#;
        let violations =
            run_rules_on_file_sync(temp.path(), Grammar::Rust, source, &rules).unwrap();

        assert!(violations.is_empty());
    }

    #[tokio::test]
    async fn embedded_line_fact_rule_flags_source_lines() {
        let temp = tempdir().unwrap();
        let rules = compile_builtin_rules()
            .unwrap()
            .into_iter()
            .filter(|rule| rule.id == "RS095")
            .collect::<Vec<_>>();

        let violations = run_rules_on_file_sync(
            temp.path(),
            Grammar::Rust,
            "//// accidental\nfn main() {}\n",
            &rules,
        )
        .unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "RS095");
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[0].column, 1);
    }

    struct BuiltinRuleFixture {
        rule_id: &'static str,
        positives: &'static [&'static str],
        negatives: &'static [&'static str],
    }

    #[tokio::test]
    async fn embedded_next_query_rules_match_positive_and_negative_samples() {
        let cases = [
            BuiltinRuleFixture {
                rule_id: "RS001",
                positives: &["fn main() { let x = 0; if x >= u32::MAX {} }\n"],
                negatives: &["fn main() { let x = 0; if x > 0 {} }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS002",
                positives: &["fn main() { a = b; b = a; }\n"],
                negatives: &["fn main() { a = b; c = a; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS004",
                positives: &["fn main() { let _f = async { spawn_task() }; }\n"],
                negatives: &["fn main() { let _f = async { spawn_task().await }; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS014",
                positives: &["fn main() { let x = 3; let _ = x * 0; }\n"],
                negatives: &["fn main() { let x = 3; let _ = x * 2; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS016",
                positives: &["fn main() { if ready() {} if  ready() {} }\n"],
                negatives: &["fn main() { if ready() {} if done() {} }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS022",
                positives: &["trait T {\n#[inline]\nfn f();\n}\n"],
                negatives: &["#[inline]\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS025",
                positives: &["fn main() { let x\u{200B} = 1; }\n"],
                negatives: &["fn main() { let x = 1; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS026",
                positives: &["fn main() { for item in iter.next() {} }\n"],
                negatives: &["fn main() { while let Some(item) = iter.next() {} }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS029",
                positives: &["fn main() { let _ = mutex.lock(); }\n"],
                negatives: &["fn main() { let _guard = mutex.lock(); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS032",
                positives: &[
                    "fn main() { let _ = std::mem::replace(&mut x, std::mem::uninitialized()); }\n",
                ],
                negatives: &["fn main() { let _ = std::mem::replace(&mut x, 1); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS034",
                positives: &["fn main() { let _ = 2_32; }\n"],
                negatives: &["fn main() { let _ = 2_i32; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS038",
                positives: &[
                    "fn main() { let _ = std::os::unix::fs::PermissionsExt::from_mode(644); }\n",
                ],
                negatives: &[
                    "fn main() { let _ = std::os::unix::fs::PermissionsExt::from_mode(0o644); }\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "RS046",
                positives: &["fn main() { let _ = [foo\nbar]; }\n"],
                negatives: &["fn main() { let _ = [foo,\nbar]; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS050",
                positives: &["fn main() { for _ in 10..2 {} }\n"],
                negatives: &["fn main() { for _ in 2..10 {} }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS051",
                positives: &["fn main() { let mut x = 1; x = x; }\n"],
                negatives: &["fn main() { let mut x = 1; let y = 2; x = y; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS055",
                positives: &[
                    "fn main() { let _ = unsafe { std::mem::transmute::<_, fn()>(0usize) }; }\n",
                ],
                negatives: &[
                    "fn main() { let _ = unsafe { std::mem::transmute::<_, usize>(0usize) }; }\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "RS056",
                positives: &[
                    "fn main() { let _ = unsafe { std::mem::transmute::<_, *const u8>(0usize) }; }\n",
                ],
                negatives: &[
                    "fn main() { let _ = unsafe { std::mem::transmute::<_, *const u8>(ptr) }; }\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "RS057",
                positives: &[
                    "fn main() { let _ = unsafe { std::mem::MaybeUninit::<u8>::uninit().assume_init() }; }\n",
                ],
                negatives: &[
                    "fn main() { let _ = unsafe { std::mem::MaybeUninit::<u8>::zeroed().assume_init() }; }\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "RS058",
                positives: &["fn main() { let _ = Vec::<MaybeUninit<u8>>::with_capacity(uninit_len); }\n"],
                negatives: &["fn main() { let _ = Vec::<u8>::with_capacity(10); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS059",
                positives: &["fn main() { let _ = () == value; }\n"],
                negatives: &["fn main() { let _ = x == value; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS060",
                positives: &["fn main() { let _ = ().hash(state); }\n"],
                negatives: &["fn main() { let _ = value.hash(state); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS063",
                positives: &["fn main() { reader.read(&mut buf); }\n"],
                negatives: &["fn main() { let n = reader.read(&mut buf); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS064",
                positives: &["#[inline]\nstruct A;\n"],
                negatives: &["#[inline]\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS075",
                positives: &["fn main() { let _ = value.abs() as u32; }\n"],
                negatives: &["fn main() { let _ = value.unsigned_abs() as u32; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS079",
                positives: &["fn main() { let _ = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }; }\n"],
                negatives: &["fn main() { let _ = unsafe { std::slice::from_raw_parts(ptr, len) }; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS081",
                positives: &["fn main() { let _ = \"abc\".is_empty(); }\n"],
                negatives: &["fn main() { let s = String::new(); let _ = s.is_empty(); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS082",
                positives: &["macro_rules! m { () => { crate::foo(); } }\n"],
                negatives: &["macro_rules! m { () => { $crate::foo(); } }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS083",
                positives: &["#[cfg_attr(feature = \"cargo-clippy\", allow(dead_code))]\nfn main() {}\n"],
                negatives: &["#[cfg_attr(feature = \"lintbook\", allow(dead_code))]\nfn main() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS087",
                positives: &["mod a;\nmod a;\n"],
                negatives: &["mod a;\nmod b;\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS088",
                positives: &["#[inline]\n#[inline]\nfn f() {}\n"],
                negatives: &["#[inline]\n#[cold]\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS089",
                positives: &["///\nfn f() {}\n"],
                negatives: &["/// docs\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS090",
                positives: &["/// docs\nfn f() {}\n"],
                negatives: &["/// docs\n\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS091",
                positives: &["#[allow(dead_code)]\nfn f() {}\n"],
                negatives: &["#[allow(dead_code)]\n\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS096",
                positives: &[
                    "fn main() { let _ = unsafe { Box::from_raw(void_ptr as *mut c_void) }; }\n",
                ],
                negatives: &["fn main() { let _ = unsafe { Box::from_raw(ptr) }; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS101",
                positives: &["fn main() { let _ = base.join(\"/tmp/file\"); }\n"],
                negatives: &["fn main() { let _ = base.join(\"tmp/file\"); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS102",
                positives: &["fn main() { let _ = run_async(); }\n"],
                negatives: &["fn main() { let _future = run_async(); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS105",
                positives: &["fn main() { let _ = value.unwrap_or(Default::default()); }\n"],
                negatives: &["fn main() { let _ = value.unwrap_or(1); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS107",
                positives: &["fn main() { total += total + delta; }\n"],
                negatives: &["fn main() { total += delta; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS109",
                positives: &["fn main() { a = b = c; }\n"],
                negatives: &["fn main() { a = b; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS111",
                positives: &["fn main() { for _ in mut_end..10 {} }\n"],
                negatives: &["fn main() { for _ in end..10 {} }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS113",
                positives: &["fn main() { let _ = s.chars().nth(0); }\n"],
                negatives: &["fn main() { let _ = s.chars().next(); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS115",
                positives: &["fn main() { let _ = s.replace(\"x\", \"x\"); }\n"],
                negatives: &["fn main() { let _ = s.replace(\"x\", \"y\"); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS118",
                positives: &[r#"fn main() { let _ = "\101"; }"#],
                negatives: &[r#"fn main() { let _ = "\x41"; }"#],
            },
            BuiltinRuleFixture {
                rule_id: "RS119",
                positives: &["fn main() { let _ = path.ends_with(\".rs\"); }\n"],
                negatives: &["fn main() { let _ = path.extension() == Some(\"rs\"); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS120",
                positives: &["fn main() { perms.set_readonly(false); }\n"],
                negatives: &["fn main() { perms.set_readonly(true); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS121",
                positives: &["fn main() { unsafe { asm!(\"\", in(reg) *ptr, options(nomem)); } }\n"],
                negatives: &["fn main() { unsafe { asm!(\"\", in(reg) value, options(nomem)); } }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS123",
                positives: &["fn main() { let _ = vec![rc.clone(); 10]; }\n"],
                negatives: &["fn main() { let _ = vec![x.clone(), y.clone()]; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS125",
                positives: &["fn main() { let _ = Vec::with_capacity(vec![0; n].len()); }\n"],
                negatives: &["fn main() { let _ = Vec::<u8>::with_capacity(n); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS126",
                positives: &["#[repr(packed)]\nstruct A(u8);\n"],
                negatives: &["#[repr(C, packed)]\nstruct A(u8);\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS127",
                positives: &["fn main() { let _ = vec![0..10]; }\n"],
                negatives: &["fn main() { let _ = vec![0, 10]; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS128",
                positives: &["fn main() { let _ = std::mem::size_of::<&u8>(); }\n"],
                negatives: &["fn main() { let _ = std::mem::size_of_val(&x); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS130",
                positives: &["fn main() { a = -b; }\n"],
                negatives: &["fn main() { a -= b; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS131",
                positives: &["fn main() { cmd.arg(\"-c ls\"); }\n"],
                negatives: &["fn main() { cmd.arg(\"-c\").arg(\"ls\"); }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS132",
                positives: &["///< docs\nfn f() {}\n"],
                negatives: &["/// docs\nfn f() {}\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS133",
                positives: &["fn main() { if ready() {}\nelse {} }\n"],
                negatives: &["fn main() { if ready() {} else {} }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS138",
                positives: &["fn main() { let _ = ! value; }\n"],
                negatives: &["fn main() { let _ = !value; }\n"],
            },
            BuiltinRuleFixture {
                rule_id: "RS139",
                positives: &["fn main() { unsafe { std::ptr::swap(ptr as *mut u8, &mut value); } }\n"],
                negatives: &["fn main() { std::mem::swap(&mut left, &mut right); }\n"],
            },
        ];

        for case in cases {
            assert_builtin_rule(case.rule_id, case.positives, case.negatives).await;
        }
    }

    #[tokio::test]
    async fn embedded_python_rules_match_positive_and_negative_samples() {
        let cases = [
            BuiltinRuleFixture {
                rule_id: "PY001",
                positives: &[
                    "try:\n    risky_operation()\nexcept Exception:\n    pass\n",
                    "try:\n    operation1()\nexcept:\n    pass\n\ntry:\n    operation2()\nexcept:\n    pass\n",
                ],
                negatives: &["def safe_function():\n    return 'ok'\n"],
            },
            BuiltinRuleFixture {
                rule_id: "PY002",
                positives: &[
                    "import sys\nsys.path.append('/some/path')\n",
                    "import sys\nsys.path.insert(0, '/some/path')\n",
                    "import sys\nsys.path = ['/new/path']\n",
                ],
                negatives: &["import sys\nprint(sys.path)\n"],
            },
            BuiltinRuleFixture {
                rule_id: "PY003",
                positives: &[
                    "import os\nport = os.getenv('PORT')\n",
                    "import os\nport = os.getenv('PORT', '8080')\n",
                ],
                negatives: &[
                    "import config\nport = config.port()\n",
                    "import os\nport = os.environ.get('PORT')\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY004",
                positives: &["try:\n    risky_operation()\nexcept:\n    pass\n"],
                negatives: &[
                    "try:\n    risky_operation()\nexcept ValueError:\n    pass\n",
                    "try:\n    risky_operation()\nexcept (ValueError, TypeError):\n    pass\n",
                    "try:\n    risky_operation()\nexcept Exception as error:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY005",
                positives: &["if x == None:\n    pass\n", "if None != y:\n    pass\n"],
                negatives: &[
                    "if x is None:\n    pass\n",
                    "if y is not None:\n    pass\n",
                    "if x == 5:\n    pass\n",
                    "if value == 'None':\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY006",
                positives: &[
                    "if x == True:\n    pass\n",
                    "if result == False:\n    pass\n",
                    "if True != condition:\n    pass\n",
                    "if value != False:\n    pass\n",
                ],
                negatives: &[
                    "if x:\n    pass\n",
                    "if not y:\n    pass\n",
                    "if x == 5:\n    pass\n",
                    "if result == 'True':\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY007",
                positives: &[
                    "if not x in items:\n    pass\n",
                    "if not (y in collection):\n    pass\n",
                ],
                negatives: &[
                    "if x not in items:\n    pass\n",
                    "if not x == y:\n    pass\n",
                    "if not x > 5:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY008",
                positives: &[
                    "if not x is None:\n    pass\n",
                    "if not (result is expected):\n    pass\n",
                ],
                negatives: &[
                    "if x is not None:\n    pass\n",
                    "if not x == y:\n    pass\n",
                    "if not x in items:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY009",
                positives: &[
                    "if type(obj) == str:\n    pass\n",
                    "if type(value) is int:\n    pass\n",
                    "if type(data) != list:\n    pass\n",
                    "if type(result) is not dict:\n    pass\n",
                    "if str == type(text):\n    pass\n",
                ],
                negatives: &[
                    "if isinstance(obj, str):\n    pass\n",
                    "if obj == 'hello':\n    pass\n",
                    "if len(obj) == 5:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY010",
                positives: &[
                    "add = lambda x, y: x + y\n",
                    "process = lambda data: data.strip().lower() if data else ''\n",
                    "func1 = func2 = lambda x: x * 2\n",
                ],
                negatives: &[
                    "def add(x, y):\n    return x + y\n",
                    "sorted_items = sorted(items, key=lambda x: x.name)\n",
                    "result = add(1, 2)\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY012",
                positives: &[
                    "regex = \"\\d+\"\n",
                    "regex_word = \"\\w+\"\n",
                    "latex = \"\\alpha \\beta \\gamma\"\n",
                ],
                negatives: &[
                    "regex = r\"\\d+\"\n",
                    "text = \"Line 1\\nLine 2\\tTabbed\"\n",
                    "quote = \"He said \\\"Hello\\\"\"\n",
                    "data = br\"raw\\bytes\"\n",
                    "greeting = f\"Hello\\{name}\"\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY014",
                positives: &[
                    "text1 = f\"Hello, World!\"\n",
                    "text2 = F\"UPPERCASE F\"\n",
                    "text3 = f\"Use {{braces}} like this\"\n",
                    "path = fr\"C:\\Users\\name\"\n",
                ],
                negatives: &[
                    "name = 'Alice'\ngreeting = f\"Hello, {name}!\"\n",
                    "formatted = f\"{value:.2f}\"\n",
                    "path = fr\"C:\\Users\\{username}\"\n",
                    "text = \"Hello, World!\"\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY015",
                positives: &[
                    "data = {\"name\": \"Alice\", \"age\": 30, \"name\": \"Bob\"}\n",
                    "scores = {1: 'first', 2: 'second', 1: 'again'}\n",
                    "config = {\"debug\": True, \"verbose\": False, \"debug\": False, \"verbose\": True}\n",
                ],
                negatives: &[
                    "person = {\"name\": \"Alice\", \"age\": 30, \"city\": \"NYC\"}\n",
                    "data = {\"user\": {\"name\": \"Alice\"}, \"admin\": {\"name\": \"Bob\"}}\n",
                    "mixed = {\"1\": \"string one\", 1: \"number one\", True: \"boolean\"}\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY016",
                positives: &[
                    "assert (1, 2)\n",
                    "assert (x > 0, y < 10)\n",
                    "assert (condition,)\n",
                ],
                negatives: &[
                    "assert (x > 0)\n",
                    "assert ()\n",
                    "assert x > 0, \"x must be positive\"\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY017",
                positives: &[
                    "if x is \"hello\":\n    pass\n",
                    "if count is 0:\n    pass\n",
                    "if result is True:\n    pass\n",
                    "if items is []:\n    pass\n",
                ],
                negatives: &[
                    "if x is None:\n    pass\n",
                    "if x == \"hello\":\n    pass\n",
                    "if x is y:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY019",
                positives: &[
                    "if (1, 2):\n    pass\n",
                    "if x < 0:\n    pass\nelif (x > 0, x < 10):\n    pass\n",
                    "if (condition,):\n    pass\n",
                ],
                negatives: &[
                    "if (x > 0):\n    pass\n",
                    "if ():\n    pass\n",
                    "if x > 0:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY020",
                positives: &[
                    "def my_function():\n    if condition:\n        break\n",
                    "for item in items:\n    def inner():\n        break\n",
                ],
                negatives: &[
                    "for item in items:\n    if condition:\n        break\n",
                    "def process_items():\n    for item in items:\n        break\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY021",
                positives: &[
                    "def my_function():\n    if condition:\n        continue\n",
                    "for item in items:\n    def inner():\n        continue\n",
                ],
                negatives: &[
                    "for item in items:\n    if should_skip(item):\n        continue\n",
                    "def process_items():\n    for item in items:\n        continue\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY022",
                positives: &[
                    "if condition:\n    yield value\n",
                    "class MyClass:\n    if debug:\n        yield 'debug'\n",
                ],
                negatives: &[
                    "def my_generator():\n    yield 42\n",
                    "async def async_generator():\n    yield 1\n",
                    "generator_lambda = lambda: (yield 42)\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY023",
                positives: &[
                    "if condition:\n    return value\n",
                    "class MyClass:\n    if debug:\n        return 'debug'\n",
                ],
                negatives: &[
                    "def my_function():\n    return 'normal'\n",
                    "async def async_function():\n    return 'result'\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY024",
                positives: &[
                    "try:\n    risky_operation()\nexcept:\n    pass\nexcept ValueError:\n    pass\n",
                    "try:\n    operation()\nexcept:\n    pass\nexcept Exception as error:\n    pass\n",
                    "try:\n    another_operation()\nexcept Exception:\n    pass\nexcept TypeError:\n    pass\n",
                ],
                negatives: &[
                    "try:\n    risky_operation()\nexcept ValueError:\n    pass\nexcept Exception:\n    pass\nexcept:\n    pass\n",
                    "try:\n    simple_operation()\nexcept:\n    pass\n",
                    "try:\n    operation()\nexcept Exception as error:\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY025",
                positives: &[
                    "def my_method():\n    raise NotImplemented\n",
                    "def my_method():\n    raise NotImplemented()\n",
                    "try:\n    something()\nexcept Exception as error:\n    raise NotImplemented from error\n",
                ],
                negatives: &[
                    "def my_method():\n    raise NotImplementedError()\n",
                    "def test():\n    raise ValueError('Invalid value')\n",
                    "result = NotImplemented\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY026",
                positives: &[
                    "class BadClass:\n    def __init__(self):\n        return self.value\n",
                    "class ConditionalReturn:\n    def __init__(self, data):\n        if data is None:\n            return\n",
                ],
                negatives: &[
                    "class GoodClass:\n    def __init__(self):\n        self.value = 42\n",
                    "class WithNew:\n    def __new__(cls, value):\n        return None\n",
                    "class MethodsWithReturns:\n    def get_value(self):\n        return self.value\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY027",
                positives: &[
                    "def outer():\n    x = 10\n    def inner():\n        global x\n        nonlocal x\n",
                    "def outer():\n    value = 42\n    def inner():\n        nonlocal value\n        global value\n",
                ],
                negatives: &[
                    "def only_global():\n    def inner():\n        global global_var\n",
                    "def different_variables():\n    local_var = 10\n    def inner():\n        global global_var\n        nonlocal local_var\n",
                    "def separate_functions():\n    value = 42\n    def func1():\n        global value\n    def func2():\n        nonlocal value\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY028",
                positives: &[
                    "def bad_function():\n    for i in range(10):\n        try:\n            process(i)\n        finally:\n            continue\n",
                    "async def bad_async():\n    for i in range(10):\n        try:\n            await process(i)\n        finally:\n            continue\n",
                ],
                negatives: &[
                    "def good_function():\n    for i in range(10):\n        try:\n            continue\n        finally:\n            cleanup()\n",
                    "def function_with_break():\n    try:\n        process()\n    finally:\n        break\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY029",
                positives: &[
                    "class BadClass(BaseClass, BaseClass):\n    pass\n",
                    "class AttributeAccess(package.module.Class, SomeOther, package.module.Class):\n    pass\n",
                ],
                negatives: &[
                    "class GoodClass(BaseClass):\n    pass\n",
                    "class MultipleUnique(A, B, C):\n    pass\n",
                    "class WithMetaclass(Base, metaclass=Meta):\n    pass\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY030",
                positives: &[
                    "__all__ = ['valid_string', 123, 'another_valid']\n",
                    "__all__ = ('item1', 'item2')\n",
                    "__all__ = ['valid', variable_name]\n",
                    "__all__ = [f'formatted_{var}']\n",
                ],
                negatives: &[
                    "__all__ = ['public_function', 'PublicClass']\n",
                    "__all__ = []\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY031",
                positives: &[
                    "def my_function():\n    __all__ = ['bad']\n",
                    "__all__ = ['initial']\n__all__.append('dynamic')\n",
                    "__all__ = ['start']\n__all__ += ['added']\n",
                    "if some_condition:\n    __all__ = ['conditional']\n",
                ],
                negatives: &[
                    "__all__ = ['public_function', 'PublicClass']\n",
                    "__all__ = []\n",
                ],
            },
            BuiltinRuleFixture {
                rule_id: "PY032",
                positives: &[
                    "raise\n",
                    "def bad_function():\n    raise\n",
                    "try:\n    operation()\nfinally:\n    raise\n",
                ],
                negatives: &[
                    "try:\n    risky_operation()\nexcept ValueError:\n    raise\n",
                    "def function():\n    raise ValueError('not bare')\n",
                ],
            },
        ];

        for case in cases {
            assert_builtin_rule_for_grammar(
                Grammar::Python,
                case.rule_id,
                case.positives,
                case.negatives,
            )
            .await;
        }
    }

    #[test]
    fn config_can_disable_embedded_builtin_rules_by_name() {
        let temp = tempdir().unwrap();
        let mut config = LintbookConfig::new(vec!["rust"]);
        let mut rust_lints = HashMap::new();
        rust_lints.insert("eq-op".to_string(), false);
        config.lints.insert("rust".to_string(), rust_lints);

        let rules = load_all_rules(temp.path(), &config).unwrap();
        assert!(!rules.iter().any(|rule| rule.name == "eq-op"));
        assert!(rules.iter().any(|rule| rule.name == "four-forward-slashes"));
    }
}
