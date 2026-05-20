# Built-In Rule Migration

The old Rust rule crate has been removed after conversion. Built-ins now live as embedded Markdown intent plus Datafox query assets in this crate.

## Verification Protocol

For each migrated rule:

1. Copy the old rule id and display name.
2. Write `builtin/<lang>/<id>-<name>.md` with only `id`, `lang`, and the intent body.
3. Write `builtin/<lang>/<id>-<name>.df` as a Datafox query set.
4. Add the asset to `BUILTIN_RULES`.
5. Add at least one positive and one negative fixture check, or a focused crate test when the rule exercises new fact schema.
6. Run `cargo test -p lintbook-rules --all-targets` and then `cargo test --workspace --all-targets`.

## Rust Rules

All Rust rules from the removed `lintbook-lang-rust` crate have a converted built-in rule asset.

Status values:

- `converted-enabled`: embedded and run by `lintbook check`.
- `pending-query`: likely expressible with the current facts and builtins.
- `pending-facts`: needs more precise facts, sibling/order helpers, token facts, or richer literal analysis.

| Rule | Name | Status | Notes |
| --- | --- | --- | --- |
| RS001 | absurd-extreme-comparisons | converted-enabled | Uses comparison operand helpers plus extreme value facts. |
| RS002 | almost-swapped | converted-enabled | Uses assignment and adjacent code sibling facts. |
| RS003 | approx-constant | converted-enabled | Initial regex-based float literal version. |
| RS004 | async-yields-async | converted-enabled | Text query over async blocks returning future-like calls. |
| RS013 | eq-op | converted-enabled | Uses binary operator and equal operand text. |
| RS014 | erasing-op | converted-enabled | Binary expression pattern over erasing operands. |
| RS016 | ifs-same-cond | converted-enabled | Uses adjacent statement facts plus collapsed condition text. |
| RS022 | inline-fn-without-body | converted-enabled | Function signature item plus inline attribute pattern. |
| RS025 | invisible-characters | converted-enabled | Uses synthetic invisible-character facts with exact source locations. |
| RS026 | iter-next-loop | converted-enabled | For-loop iterable call pattern. |
| RS027 | iter-skip-zero | converted-enabled | Initial call-text pattern. |
| RS028 | iterator-step-by-zero | converted-enabled | Initial call-text pattern. |
| RS029 | let-underscore-lock | converted-enabled | Let pattern plus lock-like call text. |
| RS032 | mem-replace-with-uninit | converted-enabled | Call chain pattern. |
| RS034 | mistyped-literal-suffixes | converted-enabled | Uses derived mistyped literal suffix facts. |
| RS035 | modulo-one | converted-enabled | Semicolon-separated query set. |
| RS038 | non-octal-unix-permissions | converted-enabled | Permission call plus decimal-looking literal pattern. |
| RS041 | option-env-unwrap | converted-enabled | Initial call-text pattern. |
| RS046 | possible-missing-comma | converted-enabled | Uses array element layout helper facts. |
| RS050 | reversed-empty-ranges | converted-enabled | Uses range bound and parsed integer literal facts. |
| RS051 | self-assignment | converted-enabled | Assignment left/right text equality. |
| RS054 | suspicious-splitn | converted-enabled | Semicolon-separated query set. |
| RS055 | transmute-null-to-fn | converted-enabled | Transmute call text plus null/function-pointer context. |
| RS056 | transmuting-null | converted-enabled | Transmute call text plus null argument. |
| RS057 | uninit-assumed-init | converted-enabled | MaybeUninit uninit assume_init call chain. |
| RS058 | uninit-vec | converted-enabled | Vec initialization text pattern. |
| RS059 | unit-cmp | converted-enabled | Uses comparison and unit-like expression facts. |
| RS060 | unit-hash | converted-enabled | Uses unit-like expression facts over hash method/call shapes. |
| RS063 | unused-io-amount | converted-enabled | Expression statement over read/write calls. |
| RS064 | useless-attribute | converted-enabled | Uses attribute name plus adjacent target facts. |
| RS065 | vec-resize-to-zero | converted-enabled | Initial call-text pattern. |
| RS075 | cast-abs-to-unsigned | converted-enabled | Cast expression over abs call. |
| RS079 | cast-slice-from-raw-parts | converted-enabled | Call/cast text pattern. |
| RS081 | const-is-empty | converted-enabled | Literal/array/constant-looking is_empty pattern. |
| RS082 | crate-in-macro-def | converted-enabled | Macro definition crate token pattern. |
| RS083 | deprecated-clippy-cfg-attr | converted-enabled | Attribute text pattern. |
| RS087 | duplicate-mod | converted-enabled | Uses module declaration name facts and node order. |
| RS088 | duplicated-attributes | converted-enabled | Uses attribute ownership facts and node order. |
| RS089 | empty-docs | converted-enabled | Comment text pattern. |
| RS090 | empty-line-after-doc-comments | converted-enabled | Uses sibling line-gap facts. |
| RS091 | empty-line-after-outer-attr | converted-enabled | Uses attribute name and sibling line-gap facts. |
| RS092 | empty-loop | converted-enabled | Initial loop body regex. |
| RS095 | four-forward-slashes | converted-enabled | Uses synthetic `line` facts. |
| RS096 | from-raw-with-void-ptr | converted-enabled | from_raw call/cast text pattern. |
| RS101 | join-absolute-paths | converted-enabled | Call argument string pattern. |
| RS102 | let-underscore-future | converted-enabled | Let pattern plus future-like call text. |
| RS105 | manual-unwrap-or-default | converted-enabled | unwrap_or/unwrap_or_else default pattern. |
| RS107 | misrefactored-assign-op | converted-enabled | Compound assignment expression pattern. |
| RS109 | multi-assignments | converted-enabled | Uses nested assignment operand facts. |
| RS111 | mut-range-bound | converted-enabled | Uses range bound facts and mutable-looking identifier names. |
| RS113 | needless-character-iteration | converted-enabled | Call chain pattern. |
| RS115 | no-effect-replace | converted-enabled | Replace call with identical string args. |
| RS118 | octal-escapes | converted-enabled | String/char literal regex. |
| RS119 | path-ends-with-ext | converted-enabled | Method-call pattern. |
| RS120 | permissions-set-readonly-false | converted-enabled | Method-call pattern. |
| RS121 | pointers-in-nomem-asm-block | converted-enabled | asm macro text pattern. |
| RS123 | rc-clone-in-vec-init | converted-enabled | Vec macro text pattern. |
| RS125 | repeat-vec-with-capacity | converted-enabled | Vec repeat/capacity text pattern. |
| RS126 | repr-packed-without-abi | converted-enabled | Attribute text pattern. |
| RS127 | single-range-in-vec-init | converted-enabled | Vec macro text pattern. |
| RS128 | size-of-ref | converted-enabled | Call text pattern. |
| RS130 | suspicious-assignment-formatting | converted-enabled | Uses assignment expression source text spacing regex. |
| RS131 | suspicious-command-arg-space | converted-enabled | Command arg string pattern. |
| RS132 | suspicious-doc-comments | converted-enabled | Comment text pattern. |
| RS133 | suspicious-else-formatting | converted-enabled | Uses if-expression source text spacing regexes. |
| RS138 | suspicious-unary-op-formatting | converted-enabled | Unary expression text pattern. |
| RS139 | swap-ptr-to-ref | converted-enabled | Swap call pattern. |

## Python Rules

The parked Python crate remains in the tree while rules are converted gradually.

| Rule | Name | Status | Notes |
| --- | --- | --- | --- |
| PY001 | no-try-catch | converted-enabled | Uses Python try statement nodes. |
| PY002 | no-sys-path-modification | converted-enabled | Uses sys.path call/assignment shapes. |
| PY003 | no-os-getenv | converted-enabled | Uses Python call/attribute tree-sitter fields. |
| PY004 | no-bare-except | converted-enabled | Uses except-clause text matching. |
| PY005 | none-comparison | converted-enabled | Uses comparison operator and `none` node facts. |
| PY006 | true-false-comparison | converted-enabled | Uses comparison operator and boolean node facts. |
| PY007 | not-in-test | converted-enabled | Uses `not_operator` over direct or parenthesized `in` comparisons. |
| PY008 | not-is-test | converted-enabled | Uses `not_operator` over direct or parenthesized `is` comparisons. |
| PY009 | type-comparison | converted-enabled | Uses comparison operators over `type(...)` call children. |
| PY010 | lambda-assignment | converted-enabled | Uses assignment right-hand `lambda` field. |
| PY012 | invalid-escape-sequence | converted-enabled | Uses string-content invalid backslash nodes while skipping raw/f-string prefixes. |
| PY014 | f-string-missing-placeholders | converted-enabled | Uses f-string prefix and placeholder text matching. |
| PY015 | multi-value-repeated-key-literal | converted-enabled | Uses dictionary pair key text equality within the same dictionary. |
| PY016 | assert-tuple | converted-enabled | Uses assert statement tuple child facts. |
| PY017 | is-literal | converted-enabled | Uses identity comparison operators with literal child kinds. |
| PY019 | if-tuple | converted-enabled | Uses if/elif tuple child facts. |
| PY020 | break-outside-loop | converted-enabled | Uses reusable Python outside-loop context facts. |
| PY021 | continue-outside-loop | converted-enabled | Uses reusable Python outside-loop context facts. |
| PY022 | yield-outside-function | converted-enabled | Uses reusable Python outside-function context facts. |
| PY023 | return-outside-function | converted-enabled | Uses reusable Python outside-function context facts. |
| PY024 | default-except-not-last | converted-enabled | Uses direct try/except child order and except-clause text. |
| PY025 | raise-not-implemented | converted-enabled | Uses raise statement child shapes. |
| PY026 | return-in-init | converted-enabled | Uses function name fields and descendant return statements. |
| PY027 | nonlocal-and-global | converted-enabled | Uses reusable same-function declaration facts. |
| PY028 | continue-in-finally | converted-enabled | Uses finally-clause descendants. |
| PY029 | duplicate-bases | converted-enabled | Uses class argument list base text equality. |
| PY030 | invalid-all-object | converted-enabled | Uses `__all__` assignment value and element kind facts. |
| PY031 | invalid-all-format | converted-enabled | Uses `__all__` assignment parents and mutation call shapes. |
| PY032 | misplaced-bare-raise | converted-enabled | Uses reusable Python outside-except context facts. |
| PY033 | unused-import | converted-enabled | Uses import binding and name-use facts. |
| PY034 | late-future-import | converted-enabled | Uses module-level future import ordering facts. |

## Elixir Rules

The parked Elixir crate remains in the tree while rules are converted gradually.

| Rule | Name | Status | Notes |
| --- | --- | --- | --- |
| EX1001 | exception-names | converted-enabled | Uses exception module text patterns. |
| EX1002 | line-endings | converted-enabled | Uses reusable line-ending style facts. |
| EX1003 | multi_alias_import_require_use | converted-enabled | Uses reusable namespace statement facts. |
| EX1005 | space_around_operators | converted-enabled | Uses binary operator source text patterns. |
| EX1006 | space_in_parentheses | converted-enabled | Uses line text bracket spacing patterns. |
| EX1007 | tabs_or_spaces | converted-enabled | Uses reusable line indentation facts. |
| EX1008 | unused_variable_names | converted-enabled | Uses identifier self-joins over underscore names. |
| EX3001 | iex-pry | converted-enabled | Uses call function text matching. |
| EX3002 | io-inspect | converted-enabled | Uses call function text plus import detection. |
| EX3003 | variable-names | converted-enabled | Uses identifier text patterns. |
| EX3010 | trailing-whitespace | converted-enabled | Uses synthetic line facts. |
| EX3011 | semicolons | converted-enabled | Uses tree-sitter semicolon token facts. |
| EX4001 | unsafe-to-atom | converted-enabled | Uses call function text matching. |
| EX5001 | function-names | converted-enabled | Uses def call signature text matching. |
| EX5002 | module-names | converted-enabled | Uses defmodule argument text matching. |
| EX5003 | unsafe_exec | converted-enabled | Uses unsafe call function text plus argument heuristics. |
| EX5006 | dbg | converted-enabled | Uses call function text matching. |
