# TreeLint Lint Rules

This document provides a comprehensive overview of all lint rules implemented in TreeLint.

## All Implemented Rules

| Language | Status | ID | Name | Description |
|----------|--------|----|----- |-------------|
| Elixir | ✅ | EX1001 | exception-names | Exception module names should follow consistent naming pattern |
| Elixir | ✅ | EX1002 | line-endings | Consistent line endings across all files |
| Elixir | ✅ | EX3001 | iex-pry | Detect leftover IEx.pry/0 calls |
| Elixir | ✅ | EX3002 | io-inspect | Detect leftover IO.inspect/1 calls |
| Elixir | ✅ | EX3003 | variable-names | Enforce snake_case for variable names |
| Elixir | ✅ | EX3010 | trailing-whitespace | No trailing whitespace at end of lines |
| Elixir | ✅ | EX4001 | unsafe-to-atom | Prevent creating atoms dynamically from external sources |
| Elixir | ✅ | EX5001 | function-names | Enforce snake_case for function names |
| Elixir | ✅ | EX5002 | module-names | Enforce PascalCase for module names |
| Elixir | ✅ | EX5006 | dbg | Detect leftover dbg/0,1,2 calls (Elixir 1.14+) |
| Python | ✅ | PY001 | no-try-catch | Disallow try/except statements |
| Python | ✅ | PY002 | no-sys-path-modification | Disallow modification of sys.path |
| Python | ✅ | PY003 | no-os-getenv | Disallow os.getenv usage |
| Python | ✅ | PY004 | no-bare-except | Disallow bare except clauses |
| Python | ✅ | PY005 | none-comparison | Comparison to None should be 'cond is None' |
| Python | ✅ | PY006 | true-false-comparison | Comparison to True/False should be 'if cond:' or 'if not cond:' |
| Python | ✅ | PY007 | not-in-test | Use 'not in' instead of 'not x in y' |
| Python | ✅ | PY008 | not-is-test | Use 'is not' instead of 'not x is y' |
| Python | ✅ | PY009 | type-comparison | Use isinstance() instead of type() comparison |
| Python | ✅ | PY010 | lambda-assignment | Do not assign lambda expressions, use def |
| Python | ✅ | PY012 | invalid-escape-sequence | Use raw strings for regex patterns and escape sequences |
| Python | ✅ | PY014 | f-string-missing-placeholders | F-strings without placeholders should be regular strings |
| Python | ✅ | PY015 | multi-value-repeated-key-literal | Dictionary contains duplicate keys |
| Python | ✅ | PY016 | assert-tuple | Assert test is a non-empty tuple |
| Python | ✅ | PY020 | break-outside-loop | Break statement outside loop |
| Python | ✅ | PY021 | continue-outside-loop | Continue statement outside loop |
| Python | ✅ | PY022 | yield-outside-function | Yield statement outside function |
| Python | ✅ | PY023 | return-outside-function | Return statement outside function |
| Python | ✅ | PY024 | default-except-not-last | Default except must be last |
| Python | ✅ | PY025 | raise-not-implemented | Use NotImplementedError not NotImplemented |
| Python | ✅ | PY026 | return-in-init | Return statement in __init__ |
| Python | ✅ | PY027 | nonlocal-and-global | Name is both nonlocal and global |
| Python | ✅ | PY028 | continue-in-finally | Continue not supported in finally |
| Python | ✅ | PY029 | duplicate-bases | Duplicate bases in class definition |
| Rust | ✅ | RS001 | absurd-extreme-comparisons | Checks for comparisons with extreme values that are always true or false |
| Rust | ✅ | RS002 | almost-swapped | Detects patterns like `foo = bar; bar = foo` that look like attempted swaps |
| Rust | ✅ | RS003 | approx-constant | Checks for hardcoded approximations of mathematical constants |
| Rust | ✅ | RS004 | async-yields-async | Detects async blocks that return awaitables without awaiting them |
| Rust | ✅ | RS013 | eq-op | Detects equal operands in binary operations |
| Rust | ✅ | RS014 | erasing-op | Detects operations that always return a constant value regardless of operands |
| Rust | ✅ | RS016 | ifs-same-cond | Detects consecutive if statements with the same condition |
| Rust | ✅ | RS022 | inline-fn-without-body | Detects inline attributes on trait methods or functions without bodies |
| Rust | ✅ | RS025 | invisible-characters | Detects invisible Unicode characters that may cause confusion |
| Rust | ✅ | RS026 | iter-next-loop | Detects for loops iterating over iterator.next() calls |
| Rust | ✅ | RS027 | iter-skip-zero | Detects calls to `.skip(0)` on iterators |
| Rust | ✅ | RS028 | iterator-step-by-zero | Detects calls to `.step_by(0)` on iterators |
| Rust | ✅ | RS029 | let-underscore-lock | Detects `let _ = lock` patterns that immediately drop locks |
| Rust | ✅ | RS032 | mem-replace-with-uninit | Detects `mem::replace` with `mem::uninitialized()` which is dangerous |
| Rust | ✅ | RS034 | mistyped-literal-suffixes | Warns for mistyped suffix in literals |
| Rust | ✅ | RS035 | modulo-one | Checks for getting the remainder of integer division by one or minus one |
| Rust | ✅ | RS038 | non-octal-unix-permissions | Checks for non-octal values used to set Unix file permissions |
| Rust | ✅ | RS041 | option-env-unwrap | Checks for usage of `option_env!(...).unwrap()` and suggests usage of the `env!` macro |
| Rust | ✅ | RS046 | possible-missing-comma | Checks for possible missing comma in an array |
| Rust | ✅ | RS050 | reversed-empty-ranges | Checks for reversed range literals that result in empty ranges |
| Rust | ✅ | RS051 | self-assignment | Checks for assignments where the left and right sides are identical |
| Rust | ✅ | RS054 | suspicious-splitn | Checks for suspicious splitn calls with n=0 or n=1 |
| Rust | ✅ | RS059 | unit-cmp | Checks for comparisons with unit type () |
| Rust | ✅ | RS075 | cast-abs-to-unsigned | Checks for casting the result of abs() to unsigned types |
| Rust | ✅ | RS088 | duplicated-attributes | Checks for duplicate attributes on items |
| Rust | ✅ | RS092 | empty-loop | Checks for empty loop bodies |
| Rust | ✅ | RS095 | four-forward-slashes | Checks for //// comments which may be unintentional |
| SQL | ✅ | SQL001 | table-aliasing-style | Table aliases should use explicit AS keyword |
| SQL | ✅ | SQL002 | column-aliasing-style | Column aliases should use explicit AS keyword |
| SQL | ✅ | SQL003 | keyword-capitalization | SQL keywords should be consistently capitalized |
| SQL | ✅ | SQL004 | identifier-capitalization | Database identifiers should follow consistent capitalization |
| SQL | ✅ | SQL005 | not-equal-operator-consistency | Use consistent not-equal operators throughout the codebase |
| SQL | ✅ | SQL006 | coalesce-over-ifnull | Use COALESCE instead of IFNULL or NVL for better SQL standard compliance |
| SQL | ✅ | SQL007 | is-null-vs-equal-null | Use IS NULL instead of = NULL for null comparisons |
| SQL | ✅ | SQL008 | explicit-union-all | Use explicit UNION ALL or UNION DISTINCT instead of plain UNION |
| SQL | ✅ | SQL009 | explicit-join-types | Use explicit JOIN types (INNER JOIN, LEFT JOIN) instead of implicit joins |
| SQL | ✅ | SQL010 | remove-redundant-else-null | Remove redundant ELSE NULL from CASE statements |
| SQL | ✅ | SQL011 | references-in-from | All table references must be defined in FROM clause |
| SQL | ✅ | SQL012 | keywords-as-identifiers | Avoid using SQL keywords as unquoted identifiers |
| SQL | ✅ | SQL013 | no-sp-prefix | Avoid SP_ prefix for user-defined stored procedures (T-SQL) |
| SQL | ✅ | SQL014 | complex-expressions-need-aliases | Complex expressions in SELECT should have aliases |
| SQL | ✅ | SQL015 | unique-table-aliases | Table aliases must be unique within a query |
| SQL | ✅ | SQL016 | consistent-count-syntax | Use consistent COUNT syntax throughout the codebase |
| SQL | ✅ | SQL017 | distinct-group-by-conflict | DISTINCT and GROUP BY serve similar purposes and may be redundant |
| SQL | ✅ | SQL018 | trailing-commas | Use consistent trailing comma style in SELECT clauses |
| SQL | ✅ | SQL019 | unused-table-aliases | Remove unused table aliases to improve code clarity |
| SQL | ✅ | SQL020 | semicolon-placement | Consistent semicolon placement and usage |
| SQL | ✅ | SQL021 | consistent-order-by-directions | Use explicit ASC/DESC in ORDER BY clauses for consistency |
| SQL | ✅ | SQL022 | quote-style-consistency | Use consistent quote style for string literals |
| SQL | ✅ | SQL023 | type-casting-style | Use consistent type casting style |
| SQL | ✅ | SQL024 | wildcard-column-ambiguity | Avoid SELECT * in joins or when column count matters |
| SQL | ✅ | SQL025 | unnecessary-quoted-identifiers | Remove unnecessary quotes around identifiers |
| SQL | ✅ | SQL026 | table-alias-length | Table aliases should be meaningful and not too short |
| SQL | ✅ | SQL027 | avoid-aliases-in-ctes | Avoid unnecessary table aliases in CTE definitions |
| SQL | ✅ | SQL028 | column-names-in-group-order-by | Use column names instead of positional references in GROUP BY/ORDER BY |
| SQL | ✅ | SQL029 | quoted-literals | Use consistent quote style for string literals |
| SQL | ✅ | SQL030 | distinct-values-in-clause | IN clauses should not contain duplicate values |
| SQL | ✅ | SQL031 | leading-whitespace | Lines should not have unnecessary leading whitespace |
| SQL | ✅ | SQL032 | trailing-whitespace | Lines should not have trailing whitespace |
| SQL | ✅ | SQL033 | nested-case-statements | Avoid deeply nested CASE statements |
| SQL | ✅ | SQL034 | boolean-value-expressions | Boolean value expressions should be simplified |
| SQL | ✅ | SQL035 | comparison-operators | Consistent comparison operator usage |
| SQL | ✅ | SQL036 | line-length | Lines should not exceed maximum length |
| SQL | ✅ | SQL037 | join-condition-order | JOIN conditions should follow consistent ordering |
| SQL | ✅ | SQL038 | nested-subqueries | Avoid deeply nested subqueries |
| SQL | ✅ | SQL039 | qualification-consistency | Table/column qualification should be consistent |
| SQL | ✅ | SQL040 | consistent-table-references | Table references should follow consistent style |
| SQL | ✅ | SQL041 | commas-placement | Comma placement should be consistent |
| SQL | ✅ | SQL042 | line-breaks-in-clauses | Line breaks in SQL clauses should be consistent |
| SQL | ✅ | SQL043 | cte-vs-subquery | Prefer CTEs over subqueries where appropriate |
| SQL | ✅ | SQL044 | wildcard-in-count | Avoid wildcards in COUNT functions |
| SQL | ✅ | SQL045 | operators-spacing | Operator spacing should be consistent |
| SQL | ✅ | SQL046 | unnecessary-distinct | Remove unnecessary DISTINCT keywords |
| SQL | ✅ | SQL047 | undefined-functions | Check for undefined function calls |
| SQL | ✅ | SQL048 | performance-anti-patterns | Detect SQL performance anti-patterns |
| SQL | ✅ | SQL049 | security-patterns | Detect SQL security issues |
| SQL | ✅ | SQL050 | data-type-best-practices | Data type usage best practices |
| SQL | ✅ | SQL051 | transaction-patterns | Transaction usage patterns |
| SQL | ✅ | SQL052 | index-usage-hints | Index usage optimization hints |
| SQL | ✅ | SQL053 | schema-validation | Schema validation rules |
| SQL | ✅ | SQL054 | query-complexity | Query complexity analysis |
| SQL | ✅ | SQL055 | temporal-patterns | Temporal data handling patterns |
| SQL | ✅ | SQL056 | error-handling | Error handling patterns |

## Summary

**Total Rules Implemented: 107**
- **Elixir**: 10 rules (EX1001-EX5006)
- **Python**: 18 rules (PY001-PY029, with some gaps)
- **Rust**: 19 rules (RS001-RS095, sparse numbering)
- **SQL**: 60 rules (SQL001-SQL056, comprehensive coverage)

## Usage

To see all available lints:
```bash
treelint lints
```

To see lints for a specific language:
```bash
treelint lints --language rust
treelint lints --language python
treelint lints --language sql
treelint lints --language elixir
```

## Contributing

To add a new lint rule:

1. Choose an appropriate rule ID following the pattern: `<LANG><NUMBER>`
2. Implement the rule in the appropriate language crate
3. Add comprehensive test fixtures
4. Update this document with the new rule
5. Add the rule to the language's `lints()` function

All rules implement the `Rule` trait with:
- `id()` - Returns the rule ID (e.g., "PY001")
- `name()` - Returns the rule name (e.g., "no-try-catch")
- `description()` - Returns a description of what the rule checks
- `check()` - Performs the actual linting on a tree-sitter AST