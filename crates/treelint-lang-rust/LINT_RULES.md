# Rust Lint Rules

This document lists all Rust lint rules implemented in the `treelint-lang-rust` crate.

## Implemented Lint Rules

### RS001: Absurd Extreme Comparisons
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects comparisons with extreme values that are always true or false
- **Example**: `x >= i32::MIN` (always true), `x <= i32::MAX` (always true)
- **File**: `src/lints/rs001_absurd_extreme_comparisons.rs`

### RS002: Almost Swapped
- **Status**: ✅ Implemented  
- **Category**: correctness
- **Description**: Detects variables that appear to be swapped but one assignment uses the wrong variable
- **Example**: `a = b; b = a;` (should be `a = b; b = c;` or use `mem::swap`)
- **File**: `src/lints/rs002_almost_swapped.rs`

### RS004: Async Yields Async
- **Status**: ✅ Implemented
- **Category**: suspicious
- **Description**: Detects async functions that just return another async function call
- **Example**: `async fn foo() -> Result<()> { bar().await }` could be `fn foo() -> impl Future<Output=Result<()>>`
- **File**: `src/lints/rs004_async_yields_async.rs`

### RS013: Eq Op
- **Status**: ✅ Implemented
- **Category**: correctness  
- **Description**: Detects binary operations where both operands are identical
- **Example**: `x == x`, `a && a`, `b | b`
- **File**: `src/lints/rs013_eq_op.rs`

### RS014: Erasing Op
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects operations that erase their input (like `x * 0`)
- **Example**: `x * 0`, `y & 0`, `z | !0`
- **File**: `src/lints/rs014_erasing_op.rs`

### RS016: Ifs Same Cond
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects consecutive `if`/`else if` statements with identical conditions
- **Example**: `if x { } else if x { }` (second condition will never execute)
- **File**: `src/lints/rs016_ifs_same_cond.rs`

### RS022: Inline Fn Without Body
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects `#[inline]` attributes on functions without bodies (like trait definitions)
- **Example**: `#[inline] fn foo();` in trait definitions
- **File**: `src/lints/rs022_inline_fn_without_body.rs`

### RS025: Invisible Characters
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects invisible Unicode characters that can cause confusion
- **Example**: Zero-width spaces, RTL override characters in identifiers
- **File**: `src/lints/rs025_invisible_characters.rs`

### RS026: Iter Next Loop
- **Status**: ✅ Implemented
- **Category**: style
- **Description**: Detects manual iteration using `.next()` instead of proper for loops
- **Example**: `while let Some(x) = iter.next() { }` should use `for x in iter { }`
- **File**: `src/lints/rs026_iter_next_loop.rs`

### RS027: Iter Skip Zero
- **Status**: ✅ Implemented
- **Category**: style
- **Description**: Detects `.skip(0)` calls which are no-ops
- **Example**: `iter.skip(0)` does nothing
- **File**: `src/lints/rs027_iter_skip_zero.rs`

### RS028: Iterator Step By Zero
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects `.step_by(0)` which panics
- **Example**: `(0..10).step_by(0)` will panic at runtime
- **File**: `src/lints/rs028_iterator_step_by_zero.rs`

### RS029: Let Underscore Lock
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects `let _ = mutex.lock()` which immediately releases the lock
- **Example**: `let _ = data.lock();` should be `let _guard = data.lock();`
- **File**: `src/lints/rs029_let_underscore_lock.rs`

### RS032: Mem Replace With Uninit
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects dangerous `mem::replace` with `mem::uninitialized()`
- **Example**: `mem::replace(&mut x, mem::uninitialized())` is unsafe
- **File**: `src/lints/rs032_mem_replace_with_uninit.rs`

### RS034: Mistyped Literal Suffixes
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects mistyped literal suffixes
- **Example**: `2_32` should be `2_i32`, `250_8` should be `250_u8`
- **File**: `src/lints/rs034_mistyped_literal_suffixes.rs`

### RS035: Modulo One
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects modulo operations by 1 or -1
- **Example**: `x % 1` (always 0), `x % -1` (can panic/overflow)
- **File**: `src/lints/rs035_modulo_one.rs`

### RS038: Non Octal Unix Permissions
- **Status**: ✅ Implemented
- **Category**: correctness
- **Description**: Detects decimal numbers used for Unix file permissions
- **Example**: `options.mode(644)` should be `options.mode(0o644)`
- **File**: `src/lints/rs038_non_octal_unix_permissions.rs`

## Pending Implementation (HIGH Feasibility)

### RS041: Option Env Unwrap
- **Status**: ⏳ Pending
- **Category**: restriction
- **Description**: Detects `option_env!().unwrap()` which can panic at runtime
- **Example**: `option_env!("VAR").unwrap()` should use `env!("VAR")` or proper error handling

### RS046: Possible Missing Comma
- **Status**: ⏳ Pending
- **Category**: correctness
- **Description**: Detects possible missing commas in array/slice literals
- **Example**: `[a b]` might be missing a comma: `[a, b]`

## Additional High-Feasibility Rules to Implement

Based on the analysis in `rust_lint_analysis.md`, the following rules are also marked as HIGH feasibility for tree-sitter implementation:

### RS003: Allow Attributes Without Reason
- **Category**: restriction
- **Description**: Detects `#[allow(...)]` attributes without reason comments

### RS005: Approx Constant  
- **Category**: correctness
- **Description**: Detects hardcoded constants that could use standard library constants

### RS006: Arithmetic Side Effects
- **Category**: restriction
- **Description**: Detects arithmetic operations that could overflow/panic

### RS007: As Conversions
- **Category**: restriction
- **Description**: Detects potentially unsafe `as` conversions

### RS008: As Underscore
- **Category**: style
- **Description**: Detects `as _` type conversions

### RS009: Assertions On Constants
- **Category**: style
- **Description**: Detects assertions on constant expressions

### RS010: Assign Op Pattern
- **Category**: style
- **Description**: Detects patterns that could use assignment operators

### RS011: Async Fn In Trait
- **Category**: restriction
- **Description**: Detects async functions in traits

### RS012: Blocks In If Conditions
- **Category**: style
- **Description**: Detects complex blocks in if conditions

And many more...

## Implementation Statistics

- **Total Analyzed**: 790 Clippy lints
- **HIGH Feasibility**: 513 lints (65%)
- **MEDIUM Feasibility**: 186 lints (24%)  
- **LOW Feasibility**: 91 lints (11%)
- **Currently Implemented**: 16 lints (3.1% of HIGH feasibility)
- **Remaining HIGH**: 497 lints

## Test Coverage

All implemented lints include comprehensive test fixtures in the `treelint-tests/fixtures/rust/` directory:

- `rs001_absurd_extreme_comparisons.rs`
- `rs002_almost_swapped.rs`
- `rs004_async_yields_async.rs`
- `rs013_eq_op.rs`
- `rs014_erasing_op.rs`
- `rs016_ifs_same_cond.rs`
- `rs022_inline_fn_without_body.rs`
- `rs025_invisible_characters.rs`
- `rs026_iter_next_loop.rs`
- `rs027_iter_skip_zero.rs`
- `rs028_iterator_step_by_zero.rs`
- `rs029_let_underscore_lock.rs`
- `rs032_mem_replace_with_uninit.rs`
- `rs034_mistyped_literal_suffixes.rs`
- `rs035_modulo_one.rs`
- `rs038_non_octal_unix_permissions.rs`

Each test fixture includes both violation cases (code that should trigger the lint) and valid cases (code that should not trigger the lint) to ensure accuracy.

## Architecture Notes

- All lints implement the `Rule` trait defined in `src/lib.rs`
- Tree traversal approach is used instead of tree-sitter queries for better compatibility
- Lints are registered in `src/lints/mod.rs`
- The crate is isolated from other language implementations to avoid dependency conflicts
- Uses workspace-level dependencies for consistency