# TreeLint Lint Rules

This document provides a comprehensive overview of all lint rules implemented and planned for TreeLint.

## SQL Rules (25 Implemented)

### Aliasing Rules (AL)
- **SQL001** ✅ - `table-aliasing-style` - Table aliases should use explicit AS keyword
- **SQL002** ✅ - `column-aliasing-style` - Column aliases should use explicit AS keyword  
- **SQL014** ✅ - `complex-expressions-need-aliases` - Complex expressions in SELECT should have aliases
- **SQL015** ✅ - `unique-table-aliases` - Table aliases must be unique within a query
- **SQL019** ✅ - `unused-table-aliases` - Remove unused table aliases to improve code clarity
- **AL06** 🔄 - Alias length constraints (min/max length validation)
- **AL07** 🔄 - Discourage all table aliases (force table.column references)
- **AL08** 🔄 - Unique column aliases (prevent duplicate column aliases)
- **AL09** 🔄 - Remove self-aliasing (table AS table)

### Capitalization Rules (CP)
- **SQL003** ✅ - `keyword-capitalization` - SQL keywords should be consistently capitalized
- **SQL004** ✅ - `identifier-capitalization` - Database identifiers should follow consistent capitalization
- **CP03** 🔄 - Function name capitalization
- **CP04** 🔄 - Boolean/null literal capitalization  
- **CP05** 🔄 - Data type capitalization

### Convention Rules (CV)
- **SQL005** ✅ - `not-equal-operator-consistency` - Use consistent not-equal operators throughout the codebase
- **SQL006** ✅ - `coalesce-over-ifnull` - Use COALESCE instead of IFNULL or NVL for better SQL standard compliance
- **SQL007** ✅ - `is-null-vs-equal-null` - Use IS NULL instead of = NULL for null comparisons
- **SQL016** ✅ - `consistent-count-syntax` - Use consistent COUNT syntax throughout the codebase
- **SQL018** ✅ - `trailing-commas` - Use consistent trailing comma style in SELECT clauses
- **SQL020** ✅ - `semicolon-placement` - Consistent semicolon placement and usage
- **SQL022** ✅ - `quote-style-consistency` - Use consistent quote style for string literals
- **SQL023** ✅ - `type-casting-style` - Use consistent type casting style
- **CV07** 🔄 - Remove statement brackets
- **CV08** 🔄 - Prefer LEFT JOIN over RIGHT JOIN
- **CV09** 🔄 - Block configurable words/patterns
- **CV12** 🔄 - JOIN ON vs WHERE conditions

### Ambiguous Rules (AM)
- **SQL008** ✅ - `explicit-union-all` - Use explicit UNION ALL or UNION DISTINCT instead of plain UNION
- **SQL009** ✅ - `explicit-join-types` - Use explicit JOIN types (INNER JOIN, LEFT JOIN) instead of implicit joins
- **SQL017** ✅ - `distinct-group-by-conflict` - DISTINCT and GROUP BY serve similar purposes and may be redundant
- **SQL021** ✅ - `consistent-order-by-directions` - Use explicit ASC/DESC in ORDER BY clauses for consistency
- **SQL024** ✅ - `wildcard-column-ambiguity` - Avoid SELECT * in joins or when column count matters
- **AM06** 🔄 - Consistent GROUP BY/ORDER BY references (by position vs by name)
- **AM07** 🔄 - Set operation column count validation
- **AM08** 🔄 - Implicit cross joins detection

### Structure Rules (ST)
- **SQL010** ✅ - `remove-redundant-else-null` - Remove redundant ELSE NULL from CASE statements
- **ST02** 🔄 - Simplify CASE statements
- **ST03** 🔄 - Unused CTEs detection
- **ST04** 🔄 - Flatten nested CASE statements
- **ST05** 🔄 - Subqueries to CTEs conversion
- **ST06** 🔄 - SELECT clause ordering
- **ST07** 🔄 - USING to ON conversion
- **ST08** 🔄 - Remove DISTINCT parentheses
- **ST09** 🔄 - Join condition ordering
- **ST10** 🔄 - Constant expressions detection
- **ST11** 🔄 - Unused joins detection

### References Rules (RF)
- **SQL011** ✅ - `references-in-from` - All table references must be defined in FROM clause
- **SQL012** ✅ - `keywords-as-identifiers` - Avoid using SQL keywords as unquoted identifiers
- **SQL025** ✅ - `unnecessary-quoted-identifiers` - Remove unnecessary quotes around identifiers
- **RF02** 🔄 - Qualify multi-table references
- **RF03** 🔄 - Consistent single-table qualification
- **RF05** 🔄 - Special characters in identifiers

### T-SQL Rules (TQ)
- **SQL013** ✅ - `no-sp-prefix` - Avoid SP_ prefix for user-defined stored procedures (T-SQL)

### Layout Rules (LT) - 🔄 Planned
- **LT01** 🔄 - Inappropriate spacing (core formatting)
- **LT02** 🔄 - Incorrect indentation
- **LT03** 🔄 - Operator position (leading vs trailing)
- **LT04** 🔄 - Comma position (leading vs trailing)
- **LT05** 🔄 - Line length enforcement
- **LT06** 🔄 - Function spacing
- **LT07** 🔄 - CTE bracket position
- **LT08** 🔄 - CTE newlines
- **LT09** 🔄 - SELECT targets layout
- **LT10** 🔄 - SELECT modifiers layout
- **LT11** 🔄 - Set operators layout
- **LT12** 🔄 - End of file formatting
- **LT13** 🔄 - Start of file formatting
- **LT14** 🔄 - Keyword newlines
- **LT15** 🔄 - Consecutive blank lines

### Jinja Rules (JJ) - 🔄 Planned
- **JJ01** 🔄 - Jinja tag whitespace padding

## Rust Rules (18 Implemented)

### Correctness Rules
- **RS001** ✅ - `absurd-extreme-comparisons` - Detect impossible comparisons with extreme values
- **RS002** ✅ - `almost-swapped` - Detect likely variable swap mistakes  
- **RS013** ✅ - `eq-op` - Detect binary operations where both operands are identical
- **RS014** ✅ - `erasing-op` - Detect operations that erase their input (like x * 0)
- **RS016** ✅ - `ifs-same-cond` - Detect consecutive if/else if with identical conditions
- **RS022** ✅ - `inline-fn-without-body` - Detect #[inline] on functions without bodies
- **RS025** ✅ - `invisible-characters` - Detect invisible Unicode characters
- **RS028** ✅ - `iterator-step-by-zero` - Detect .step_by(0) which panics
- **RS029** ✅ - `let-underscore-lock` - Detect let _ = mutex.lock() which releases immediately
- **RS032** ✅ - `mem-replace-with-uninit` - Detect dangerous mem::replace with mem::uninitialized
- **RS034** ✅ - `mistyped-literal-suffixes` - Detect mistyped literal suffixes (2_32 vs 2_i32)
- **RS035** ✅ - `modulo-one` - Detect modulo operations by 1 or -1
- **RS038** ✅ - `non-octal-unix-permissions` - Detect decimal numbers for Unix permissions
- **RS041** ✅ - `option-env-unwrap` - Detect option_env!().unwrap() which can panic at runtime
- **RS046** ✅ - `possible-missing-comma` - Detect possible missing commas in arrays/slices

### Async Rules
- **RS004** ✅ - `async-yields-async` - Async functions yielding async values

### Style Rules  
- **RS026** ✅ - `iter-next-loop` - Detect manual iteration using .next() instead of for loops
- **RS027** ✅ - `iter-skip-zero` - Detect .skip(0) calls which are no-ops

### Additional Rust Rules - 🔄 Planned (HIGH Feasibility)
- **RS003** 🔄 - `allow-attributes-without-reason` - #[allow] attributes without reason comments
- **RS005** 🔄 - `approx-constant` - Hardcoded constants that could use std library constants
- **RS006** 🔄 - `arithmetic-side-effects` - Arithmetic operations that could overflow/panic
- **RS007** 🔄 - `as-conversions` - Potentially unsafe as conversions
- **RS008** 🔄 - `as-underscore` - as _ type conversions
- **RS009** 🔄 - `assertions-on-constants` - Assertions on constant expressions
- **RS010** 🔄 - `assign-op-pattern` - Patterns that could use assignment operators
- **RS011** 🔄 - `async-fn-in-trait` - Async functions in traits
- **RS012** 🔄 - `blocks-in-if-conditions` - Complex blocks in if conditions

## Python Rules (5+ Implemented)

### Security & Best Practices
- **PY001** ✅ - `no-try-catch` - Discourage broad try/except blocks
- **PY002** ✅ - `no-sys-path-modification` - Avoid modifying sys.path
- **PY003** ✅ - `no-os-getenv` - Use environment variable alternatives
- **PY004** ✅ - `no-bare-except` - Avoid bare except clauses
- **PY005** ✅ - `none-comparison` - Use `is None` instead of `== None`

### Additional Python Rules - 🔄 Planned
- **PY006** 🔄 - Prefer f-strings over `.format()` and `%` formatting
- **PY007** 🔄 - Use `pathlib.Path` instead of `os.path`
- **PY008** 🔄 - Avoid mutable default arguments
- **PY009** 🔄 - Use `isinstance()` instead of `type() ==`
- **PY010** 🔄 - Prefer comprehensions over `map()` and `filter()`

## Elixir Rules (10 Implemented)

### Consistency Rules
- **EX1001** ✅ - `exception-names` - Exception module names should follow consistent naming pattern
- **EX1002** ✅ - `line-endings` - Consistent line endings across all files (Unix LF vs Windows CRLF)
- **EX1003** 🔄 - `multi-alias-import-require-use` - Consistent style for multi-alias vs single-alias syntax
- **EX1004** 🔄 - `parameter-pattern-matching` - Consistent variable placement in pattern matching
- **EX1005** 🔄 - `space-around-operators` - Consistent spacing around operators
- **EX1006** 🔄 - `space-in-parentheses` - Consistent spacing inside parentheses, brackets, and braces
- **EX1007** 🔄 - `tabs-or-spaces` - Consistent use of tabs or spaces for indentation
- **EX1008** 🔄 - `unused-variable-names` - Consistent naming of unused variables

### Design Rules
- **EX2001** 🔄 - `alias-usage` - Functions from nested modules should use aliases
- **EX2002** 🔄 - `duplicated-code` - Identify duplicated code blocks across multiple files
- **EX2003** 🔄 - `skip-test-without-comment` - Skipped tests must have comments explaining why
- **EX2004** 🔄 - `tag-fixme` - Identify FIXME comments that need immediate attention
- **EX2005** 🔄 - `tag-todo` - Identify TODO comments for future improvements

### Readability Rules
- **EX3001** ✅ - `iex-pry` - Detect leftover IEx.pry/0 calls
- **EX3002** ✅ - `io-inspect` - Detect leftover IO.inspect/1 calls
- **EX3003** ✅ - `variable-names` - Enforce snake_case for variable names
- **EX3004** 🔄 - `predicate-function-names` - Functions ending with ? should be predicates
- **EX3005** 🔄 - `strict-module-layout` - Enforce ordering of module parts (moduledoc, use, import, etc.)
- **EX3006** 🔄 - `alias-order` - Alphabetical ordering of alias statements
- **EX3007** 🔄 - `separate-alias-require` - Group alias and require statements separately
- **EX3008** 🔄 - `max-line-length` - Lines should not exceed maximum length
- **EX3009** 🔄 - `space-after-commas` - Use spaces after commas
- **EX3010** ✅ - `trailing-whitespace` - No trailing whitespace at end of lines
- **EX3011** 🔄 - `semicolons` - Don't use semicolons to separate statements
- **EX3012** 🔄 - `block-pipe` - Pipes should not be used with blocks like case, if
- **EX3013** 🔄 - `single-pipe` - Pipes should only be used for multiple function calls
- **EX3014** 🔄 - `with-single-clause` - Use case instead of with for single pattern matching

### Refactor Rules
- **EX4001** ✅ - `unsafe-to-atom` - Prevent creating atoms dynamically from external sources
- **EX4002** 🔄 - `filter-count` - Use Enum.count/2 instead of Enum.filter/2 |> Enum.count/1
- **EX4003** 🔄 - `map-join` - Use Enum.map_join/3 instead of Enum.map/2 |> Enum.join/2
- **EX4004** 🔄 - `append-single-item` - Avoid list ++ [item], use [item | list] with reverse
- **EX4005** 🔄 - `filter-filter` - Combine multiple Enum.filter calls into single operation
- **EX4006** 🔄 - `map-filter` - Use Enum.filter_map/3 or for comprehension instead of map |> filter
- **EX4007** 🔄 - `reject-reject` - Combine multiple Enum.reject calls into single operation
- **EX4008** 🔄 - `map-reduce` - Use Enum.map_reduce/3 instead of separate map and reduce operations
- **EX4009** 🔄 - `flat-map` - Use Enum.flat_map/2 instead of Enum.map/2 |> Enum.concat/1
- **EX4010** 🔄 - `reduce-into` - Use Enum.into/2 instead of Enum.reduce for collection building
- **EX4011** 🔄 - `sort-sort` - Combine multiple Enum.sort calls or use Enum.sort_by
- **EX4012** 🔄 - `abc-size` - Measure assignments, branches, and conditions using sqrt(A² + B² + C²)
- **EX4013** 🔄 - `cyclomatic-complexity` - Count decision points in functions
- **EX4014** 🔄 - `perceived-complexity` - Similar to cyclomatic but with adjusted weights
- **EX4015** 🔄 - `nesting` - Limit code nesting depth within functions
- **EX4016** 🔄 - `apply` - Prefer direct function calls over apply/2 when arguments are known
- **EX4017** 🔄 - `double-boolean-negation` - Avoid !!var patterns
- **EX4018** 🔄 - `unless-with-else` - unless blocks should not contain else clauses
- **EX4019** 🔄 - `variable-rebinding` - Discourage rebinding variables to the same name
- **EX4020** 🔄 - `io-puts` - Prefer Logger over IO.puts/1
- **EX4021** 🔄 - `module-dependencies` - Limit number of module dependencies
- **EX4022** 🔄 - `function-arity` - Limit number of function parameters

### Naming Rules
- **EX5001** ✅ - `function-names` - Enforce snake_case for function names
- **EX5002** ✅ - `module-names` - Enforce PascalCase for module names

### Warning Rules
- **EX5003** 🔄 - `unsafe-exec` - Prevent command injection vulnerabilities
- **EX5004** 🔄 - `leaky-environment` - Ensure environment variables are cleared when spawning processes
- **EX5005** 🔄 - `forbidden-module` - Prevent usage of specified hazardous modules
- **EX5006** ✅ - `dbg` - Detect leftover dbg/0,1,2 calls (Elixir 1.14+)
- **EX5007** 🔄 - `lazy-logging` - Ensure expensive Logger calls use lazy evaluation
- **EX5008** 🔄 - `bool-operation-on-same-values` - Detect redundant boolean operations
- **EX5009** 🔄 - `operation-with-constant-result` - Detect operations that always yield the same result
- **EX5010** 🔄 - `expensive-empty-enum-check` - Prevent Enum.count == 0, suggest Enum.empty?
- **EX5011** 🔄 - `unused-enum-operation` - Ensure Enum return values are used
- **EX5012** 🔄 - `unused-string-operation` - Ensure String return values are used
- **EX5013** 🔄 - `unused-file-operation` - Ensure File return values are used
- **EX5014** 🔄 - `unused-keyword-operation` - Ensure Keyword return values are used
- **EX5015** 🔄 - `unused-list-operation` - Ensure List return values are used
- **EX5016** 🔄 - `unused-path-operation` - Ensure Path return values are used
- **EX5017** 🔄 - `unused-regex-operation` - Ensure Regex return values are used
- **EX5018** 🔄 - `unused-tuple-operation` - Ensure Tuple return values are used

## Other Languages - 🔄 Planned

### JavaScript/TypeScript
- **JS001** 🔄 - Prefer `const` over `let` for immutable variables
- **JS002** 🔄 - Use strict equality (`===`) instead of loose equality (`==`)
- **JS003** 🔄 - Prefer template literals over string concatenation
- **TS001** 🔄 - Explicit return types for functions
- **TS002** 🔄 - Prefer `interface` over `type` for object types

### Go
- **GO001** 🔄 - Proper error handling (don't ignore errors)
- **GO002** 🔄 - Use `gofmt` formatting
- **GO003** 🔄 - Prefer `make()` with capacity for slices

### Java
- **JV001** 🔄 - Use `StringBuilder` for string concatenation in loops
- **JV002** 🔄 - Prefer enhanced for-loops over traditional for-loops
- **JV003** 🔄 - Use `Optional` instead of null checks

### C/C++
- **C001** 🔄 - Memory leak detection (missing free/delete)
- **C002** 🔄 - Buffer overflow prevention
- **CPP001** 🔄 - Use smart pointers instead of raw pointers
- **CPP002** 🔄 - Prefer `auto` for type deduction

## Legend

- ✅ **Implemented** - Rule is fully implemented and tested
- 🔄 **Planned** - Rule is planned for future implementation
- ❌ **Deprecated** - Rule has been removed or superseded

## Implementation Status

### Summary
- **SQL**: 25/40+ rules implemented (62%)
- **Rust**: 18/500+ rules implemented (3.6%)
- **Python**: 5/192 rules implemented (2.6%)
- **Elixir**: 10/67 rules implemented (14.9%)
- **Other Languages**: 0% implemented

### Total Rules
- **Implemented**: 58 rules (25 SQL + 18 Rust + 5 Python + 10 Elixir)
- **Planned**: 800+ additional rules
- **Total Planned**: 850+ rules across all languages

## Contributing

To add a new lint rule:

1. Choose an appropriate rule ID following the pattern: `<LANG><NUMBER>`
2. Implement the rule in the appropriate language crate
3. Add comprehensive test fixtures
4. Update this document with the new rule
5. Add the rule to the language's `get_lints()` function

For detailed implementation guidelines, see the project's contributing documentation.

# Lint Rules Implemented by Treelint

This document tracks all lint rules implemented in treelint, organized by language.

## Python Lints

### Implemented ✅

| Rule ID | Original ID | Name | Description | Status |
|---------|-------------|------|-------------|---------|
| PY005 | E711 | none-comparison | Use 'is' and 'is not' for None comparisons | ✅ |
| PY006 | E712 | true-false-comparison | Avoid comparison to True/False | ✅ |
| PY007 | E713 | not-in-test | Use 'not in' instead of 'not x in y' | ✅ |
| PY008 | E714 | not-is-test | Use 'is not' instead of 'not x is y' | ✅ |
| PY009 | E721 | type-comparison | Use isinstance() instead of type() comparison | ✅ |

### Pending Implementation 🚧

| Rule ID | Original ID | Name | Description | Priority |
|---------|-------------|------|-------------|----------|
| PY001 | - | no-try-catch | Disallow try/except statements | Medium |
| PY002 | - | no-sys-path-modification | Disallow modification of sys.path | Medium |
| PY003 | - | no-os-getenv | Disallow os.getenv usage | Medium |
| PY004 | - | no-bare-except | Disallow bare except clauses | Medium |
| PY010 | E731 | lambda-assignment | Do not assign lambda expressions, use def | High |
| PY011 | E741-E743 | ambiguous-names | Avoid ambiguous variable/class/function names | Medium |
| PY012 | W605 | invalid-escape-sequence | Use raw strings for escape sequences | High |
| PY013 | F404 | late-future-import | Future imports must be at beginning | Medium |
| PY014 | F541 | f-string-missing-placeholders | F-strings without placeholders | High |
| PY015 | F601 | multi-value-repeated-key-literal | Duplicate dictionary keys | High |
| PY016 | F631 | assert-tuple | Assert test is non-empty tuple | High |
| PY017 | F632 | is-literal | Use == for literal comparisons | High |
| PY018 | F633 | invalid-print-syntax | Python 2 print syntax | Medium |
| PY019 | F634 | if-tuple | If test is non-empty tuple | High |
| PY020 | F701 | break-outside-loop | Break statement outside loop | High |
| PY021 | F702 | continue-outside-loop | Continue statement outside loop | High |
| PY022 | F704 | yield-outside-function | Yield statement outside function | High |
| PY023 | F706 | return-outside-function | Return statement outside function | High |
| PY024 | F707 | default-except-not-last | Default except must be last | High |
| PY025 | F901 | raise-not-implemented | Use NotImplementedError not NotImplemented | High |
| PY026 | E0101 | return-in-init | Return statement in __init__ | High |
| PY027 | E0115 | nonlocal-and-global | Name is both nonlocal and global | High |
| PY028 | E0116 | continue-in-finally | Continue not supported in finally | High |
| PY029 | E0241 | duplicate-bases | Duplicate bases in class definition | High |
| PY030 | E0604 | invalid-all-object | Invalid object in __all__ | Medium |
| PY031 | E0704 | misplaced-bare-raise | Misplaced bare raise | High |
| PY032 | E1132 | repeated-keyword-argument | Repeated keyword argument | High |
| PY033 | E1142 | await-outside-async | Await outside async function | High |
| PY034 | E1700 | yield-from-in-async | Yield from in async function | High |
| PY035 | R0124 | comparison-with-itself | Comparison with itself | High |
| PY036 | R1711 | useless-return | Useless return at end of function | Medium |
| PY037 | R5501 | collapsible-else-if | Use elif instead of else if | Medium |
| PY038 | W0127 | self-assigning-variable | Self-assignment of variable | High |
| PY039 | W0129 | assert-on-string-literal | Assert on string literal | High |
| PY040 | W0406 | import-self | Module imports itself | Medium |
| PY041 | W0642 | self-or-cls-assignment | Invalid assignment to self or cls | High |
| PY042 | A001-A006 | builtin-shadowing | Variable/argument/attribute shadows builtin | Medium |
| PY043 | B002 | unary-prefix-increment | Unary prefix increment/decrement | Medium |
| PY044 | B003 | assignment-to-os-environ | Assignment to os.environ | Medium |
| PY045 | B006 | mutable-argument-default | Mutable default argument | High |
| PY046 | B008 | function-call-in-default-argument | Function call in default argument | High |
| PY047 | B009 | getattr-with-constant | getattr with constant attribute | Medium |
| PY048 | B010 | setattr-with-constant | setattr with constant attribute | Medium |
| PY049 | B011 | assert-false | Do not assert False | High |
| PY050 | B012 | jump-statement-in-finally | Return/break/continue in finally | High |
| PY051 | B014 | duplicate-handler-exception | Duplicate exception in handler | High |
| PY052 | B016 | raise-literal | Do not raise literals | High |
| PY053 | B018 | useless-expression | Useless expression | Medium |
| PY054 | B021 | f-string-docstring | f-string used as docstring | High |
| PY055 | B025 | duplicate-try-block-exception | Duplicate exception in try block | High |
| PY056 | B029 | except-with-empty-tuple | except with empty tuple | High |
| PY057 | B030 | except-with-non-exception-classes | except with non-exception classes | High |
| PY058 | B033 | duplicate-value | Duplicate value in set | High |
| PY059 | B904 | raise-without-from-inside-except | raise without from inside except | High |
| PY060 | S101 | assert | Use of assert | Low |
| PY061 | S102 | exec-builtin | Use of exec() | High |
| PY062 | S110 | try-except-pass | try-except-pass | Medium |
| PY063 | S112 | try-except-continue | try-except-continue | Medium |
| PY064 | S307 | suspicious-eval-usage | Use of eval | High |
| PY065 | C400-C419 | comprehensions | Unnecessary generators/comprehensions | Medium |
| PY066 | SIM101 | duplicate-isinstance-call | Duplicate isinstance() calls | Medium |
| PY067 | SIM102 | collapsible-if | Collapsible if statements | Medium |
| PY068 | SIM103 | needless-bool | Needless bool() | Medium |
| PY069 | SIM108 | if-else-block-instead-of-if-exp | Use ternary operator | Medium |
| PY070 | SIM109 | compare-with-tuple | Compare with tuple | Medium |
| PY071 | SIM110 | reimplemented-builtin | Reimplemented builtin | Medium |
| PY072 | SIM113 | enumerate-for-loop | Use enumerate() | Medium |
| PY073 | SIM115 | open-file-with-context-handler | Use context manager for open() | High |
| PY074 | SIM118 | in-dict-keys | Use 'in dict' instead of 'in dict.keys()' | Medium |
| PY075 | SIM201 | negate-equal-op | Use != instead of not == | Medium |
| PY076 | SIM202 | negate-not-equal-op | Use == instead of not != | Medium |
| PY077 | SIM208 | double-negation | Remove double negation | Medium |
| PY078 | RET501 | unnecessary-return-none | Unnecessary return None | Medium |
| PY079 | RET502 | implicit-return-value | Implicit return value | Medium |
| PY080 | RET503 | implicit-return | Missing explicit return | Medium |
| PY081 | RET504 | unnecessary-assign | Unnecessary assignment before return | Medium |
| PY082 | RET505 | superfluous-else-return | Unnecessary else after return | Medium |
| PY083 | RET506 | superfluous-else-raise | Unnecessary else after raise | Medium |
| PY084 | RET507 | superfluous-else-continue | Unnecessary else after continue | Medium |
| PY085 | RET508 | superfluous-else-break | Unnecessary else after break | Medium |
| PY086 | TRY002 | raise-vanilla-class | Create exception instance | Medium |
| PY087 | TRY003 | raise-vanilla-args | Avoid long exception messages | Medium |
| PY088 | TRY004 | type-check-without-type-error | Prefer TypeError for type checks | Medium |
| PY089 | TRY201 | verbose-raise | Avoid verbose raises | Medium |
| PY090 | TRY203 | useless-try-except | Useless try-except | Medium |
| PY091 | TRY300 | try-consider-else | Consider using try-else | Medium |
| PY092 | TRY301 | raise-within-try | Don't raise in try block | Medium |
| PY093 | TRY400 | error-instead-of-exception | Use logging.exception() | Medium |
| PY094 | TRY401 | verbose-log-message | Redundant exception info in log | Medium |
| PY095 | PTH100 | os-path-abspath | Use Path.resolve() | Medium |
| PY096 | PTH101 | os-chmod | Use Path.chmod() | Medium |
| PY097 | PTH102 | os-mkdir | Use Path.mkdir() | Medium |
| PY098 | PTH103 | os-makedirs | Use Path.mkdir(parents=True) | Medium |
| PY099 | PTH104 | os-rename | Use Path.rename() | Medium |
| PY100 | PTH105 | os-replace | Use Path.replace() | Medium |
| PY101 | PTH106 | os-rmdir | Use Path.rmdir() | Medium |
| PY102 | PTH107 | os-remove | Use Path.unlink() | Medium |
| PY103 | PTH108 | os-unlink | Use Path.unlink() | Medium |
| PY104 | PTH109 | os-getcwd | Use Path.cwd() | Medium |
| PY105 | PTH110 | os-path-exists | Use Path.exists() | Medium |
| PY106 | PTH111 | os-path-expanduser | Use Path.expanduser() | Medium |
| PY107 | PTH112 | os-path-isdir | Use Path.is_dir() | Medium |
| PY108 | PTH113 | os-path-isfile | Use Path.is_file() | Medium |
| PY109 | PTH114 | os-path-islink | Use Path.is_symlink() | Medium |
| PY110 | PTH115 | os-readlink | Use Path.readlink() | Medium |
| PY111 | PTH116 | os-stat | Use Path.stat() | Medium |
| PY112 | PTH117 | os-path-isabs | Use Path.is_absolute() | Medium |
| PY113 | PTH118 | os-path-join | Use Path() instead of join() | Medium |
| PY114 | PTH119 | os-path-basename | Use Path.name | Medium |
| PY115 | PTH120 | os-path-dirname | Use Path.parent | Medium |
| PY116 | PTH121 | os-path-samefile | Use Path.samefile() | Medium |
| PY117 | PTH122 | os-path-splitext | Use Path.stem and suffix | Medium |
| PY118 | PTH123 | builtin-open | Use Path.open() | Medium |
| PY119 | PTH124 | py-path | Use pathlib instead | Medium |
| PY120 | PTH201 | path-constructor-current-directory | Don't pass '.' to Path | Medium |
| PY121 | PTH202 | os-path-getsize | Use Path.stat().st_size | Medium |
| PY122 | PTH203 | os-path-getatime | Use Path.stat().st_atime | Medium |
| PY123 | PTH204 | os-path-getmtime | Use Path.stat().st_mtime | Medium |
| PY124 | PTH205 | os-path-getctime | Use Path.stat().st_ctime | Medium |
| PY125 | PTH206 | os-sep-split | Use Path.parts | Medium |
| PY126 | PTH207 | glob | Use Path.glob() or rglob() | Medium |
| PY127 | T201 | print | print() found | Medium |
| PY128 | T203 | pprint | pprint() found | Medium |
| PY129 | T100 | debugger | Debugger usage detected | High |
| PY130 | C901 | complex-structure | Function is too complex | Medium |
| PY131 | ERA001 | commented-out-code | Found commented-out code | Medium |
| PY132 | PERF101 | unnecessary-list-cast | Unnecessary list() cast | Medium |
| PY133 | PERF102 | incorrect-dict-iterator | Use dict.items() | Medium |
| PY134 | PERF203 | try-except-in-loop | try-except in loop | Medium |
| PY135 | PERF401 | manual-list-comprehension | Use list comprehension | Medium |
| PY136 | PERF402 | manual-list-copy | Use list.copy() | Medium |
| PY137 | PERF403 | manual-dict-comprehension | Use dict comprehension | Medium |
| PY138 | FURB101 | read-whole-file | Use Path.read_text() | Medium |
| PY139 | FURB103 | write-whole-file | Use Path.write_text() | Medium |
| PY140 | FURB105 | print-empty-string | Don't print empty string | Medium |
| PY141 | FURB110 | if-exp-instead-of-or-operator | Use or operator | Medium |
| PY142 | FURB113 | repeated-append | Use list.extend() | Medium |
| PY143 | FURB116 | f-string-number-format | Use f-string formatting | Medium |
| PY144 | FURB118 | reimplemented-operator | Use operator module | Medium |
| PY145 | FURB122 | for-loop-writes | Use writelines() | Medium |
| PY146 | FURB129 | readlines-in-for | Don't use readlines() in for | Medium |
| PY147 | FURB131 | delete-full-slice | Use clear() | Medium |
| PY148 | FURB132 | check-and-remove-from-set | Use discard() | Medium |
| PY149 | FURB136 | if-expr-min-max | Use min()/max() | Medium |
| PY150 | FURB140 | reimplemented-starmap | Use itertools.starmap() | Medium |
| PY151 | FURB142 | for-loop-set-mutations | Use set operations | Medium |
| PY152 | FURB145 | slice-copy | Use list.copy() | Medium |
| PY153 | FURB148 | unnecessary-enumerate | Remove enumerate() | Medium |
| PY154 | FURB152 | math-constant | Use math constant | Medium |
| PY155 | FURB154 | repeated-global | Combine globals | Medium |
| PY156 | FURB156 | hardcoded-string-charset | Use string module | Medium |
| PY157 | FURB157 | verbose-decimal-constructor | Simplify Decimal() | Medium |
| PY158 | FURB161 | bit-count | Use int.bit_count() | Medium |
| PY159 | FURB162 | fromisoformat-replace-z | Don't replace Z manually | Medium |
| PY160 | FURB163 | redundant-log-base | Redundant log base | Medium |
| PY161 | FURB164 | unnecessary-from-float | Remove from_float() | Medium |
| PY162 | FURB166 | int-on-sliced-str | Use int() with base | Medium |
| PY163 | FURB167 | regex-flag-alias | Use re flag name | Medium |
| PY164 | FURB168 | isinstance-type-none | Use is None check | Medium |
| PY165 | FURB169 | type-none-comparison | Use is None check | Medium |
| PY166 | FURB171 | single-item-membership-test | Use == for single item | Medium |
| PY167 | FURB177 | implicit-cwd | Omit default cwd | Medium |
| PY168 | FURB180 | meta-class-abc-meta | Use ABC | Medium |
| PY169 | FURB181 | hashlib-digest-hex | Use hexdigest() | Medium |
| PY170 | FURB187 | list-reverse-copy | Use reversed() | Medium |
| PY171 | FURB188 | slice-to-remove-prefix-or-suffix | Use removeprefix/suffix | Medium |
| PY172 | FURB189 | subclass-builtin | Subclass builtin | Medium |
| PY173 | FURB192 | sorted-min-max | Don't sort for min/max | Medium |
| PY174 | FLY002 | static-join-to-f-string | Use f-string instead of join | Medium |
| PY175 | PD002 | pandas-use-of-inplace-argument | Avoid inplace=True | Low |
| PY176 | PD003 | pandas-use-of-dot-is-null | Use .isna() instead of .isnull() | Low |
| PY177 | PD004 | pandas-use-of-dot-not-null | Use .notna() instead of .notnull() | Low |
| PY178 | PD007 | pandas-use-of-dot-ix | .ix is deprecated | Low |
| PY179 | PD008 | pandas-use-of-dot-at | Use .loc[] instead of .at[] | Low |
| PY180 | PD009 | pandas-use-of-dot-iat | Use .iloc[] instead of .iat[] | Low |
| PY181 | PD010 | pandas-use-of-dot-pivot-or-unstack | Use pivot_table() | Low |
| PY182 | PD011 | pandas-use-of-dot-values | Use .to_numpy() instead | Low |
| PY183 | PD012 | pandas-use-of-dot-read-table | Use read_csv() | Low |
| PY184 | PD013 | pandas-use-of-dot-stack | Use melt() instead | Low |
| PY185 | PD015 | pandas-use-of-pd-merge | Use DataFrame.merge() | Low |
| PY186 | PD101 | pandas-nunique-constant-series-check | nunique() on constant series | Low |
| PY187 | AIR001 | airflow-variable-name-task-id-mismatch | Task ID mismatch | Low |
| PY188 | AIR002 | airflow-dag-no-schedule-argument | DAG without schedule | Low |
| PY189 | AIR301 | airflow3-removal | Removed in Airflow 3 | Low |
| PY190 | AIR302 | airflow3-moved-to-provider | Moved to provider | Low |
| PY191 | AIR311 | airflow3-suggested-update | Suggested update | Low |
| PY192 | AIR312 | airflow3-suggested-to-move-to-provider | Suggest provider | Low |

## Summary

- **Total Python Lints**: 192
- **Implemented**: 5
- **Pending**: 187
- **Coverage**: 2.6%

## Notes

- All implemented lints include comprehensive test suites with fixtures
- Python lints are implemented in the `treelint-lang-python` crate for isolation
- Original rule IDs are preserved from tools like flake8, pylint, ruff, bandit, etc.
- Priority levels guide implementation order (High → Medium → Low)
