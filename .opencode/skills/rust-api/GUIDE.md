# Rust Project Structure Guidelines: Linting & Documentation Compliance

Below is a recommended project structure and organization that ensures code quality, doc comments are visible to documentation tooling, proper error handling, `#[non_exhaustive]` usage, consistent patterns, and type inference optimization for clarity and readability.

## Overview

Use this layout:

```
project_root/
├── Cargo.toml                          # Rust project metadata
├── src/
│   ├── main.rs                         # Entry point (use main() first)
│   ├── lib.rs                            # Reusable codebase items
│   │   └── config/                    # Config-related logic
│   │       ├── Cargo.toml             # Subpackage metadata
│   │       ├── struct/                # Public API
│   │       │   ├── pub.rs              # Struct definitions
│   │       │   ├── public.rs           # Public types (enum, struct)
│   │       │   └── private.rs          # Internal data structures
│   │       └── doc.rs                  # Documentation metadata
│   │
│   ├── config/                         # Configuration storage & logic
│   │   ├── Cargo.toml                 # Subpackage metadata
│   │   ├── data/                      # Local data (e.g., config)
│   │   │   └── pub.rs                # Public constants, structs, and API
│   │   ├── doc.rs                     # Documentation metadata
│   │   └── public.rs                  # Config structs & public functions
│   │
│   ├── test/                           # Test files (use `#[test]`)
│   │   ├── Cargo.toml                 # Test package metadata
│   │   ├── fixtures/                   # Fixture data for tests
│   │   │   └── pub.rs                # Public fixture functions and values
│   │   ├── test/                      # Test file items
│   │   │   ├── Cargo.toml             # Test package metadata
│   │   │   └── test_runner.rs          # Test runner logic
│   │   └── impl/                        # Implementation details (tests only)
│   │       └── tests/                  # Implementation unit tests
│   │           ├── Cargo.toml         # Subpackage metadata
│   │           └── impl_test.rs        # Implementation tests
│   │
│   ├── examples/                       # Examples for users
│   │   ├── src/                     # Example code
│   │   │   ├── src/main.rs          # Entry point (use main() first)
│   │   │   ├── src/lib.rs           # Common types & functions
│   │   │   └── pub.rs               # Public API & functions
│   │   └── examples/                # Example code for users
│   │       ├── Cargo.toml             # Subpackage metadata
│   │       └── example1.rs            # Example 1
│   │       └── example2.rs            # Example 2
│   │
│   └── README.md                       # Project documentation (use `readme` instead of `README.md`)
```

## Documentation Rules & Compliance

- Use the appropriate markdown syntax to ensure:
  - **Doc comments are visible** to documentation tooling.
  - No flakiness or unpredictability in documentation processes.

Use this:

````markdown
### Code Formatting

- Spaces around braces.

### Doc Comments

- Ensure `#[doc]` attributes are present.
- Use meaningful format strings in comments.

### API Documentation

- Include `#[cfg]` for code coverage.
- Use docstrings, not just `#[doc]`.

### Error Handling

- Prefer patterns over check-then-unwrap.
- Use `expect!` or pattern matching where appropriate.

### Rust Features

- Avoid unstable features.
- Prefer safe alternatives for performance-critical code.

## Code Quality Guidelines (Linting)

| Rule                                        | Description                                                                                                                                     | Clippy Lint Error Type               | Example                                                                                            |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ | -------------------------------------------------------------------------------------------------- |
| Unstable features                           | Don't use `#![unstable_features]`. If used, it must be justified.                                                                               | `clippy! unimplemented`              |
| Unsafe code                                 | Only include unsafe code if not a general-purpose optimization. Add safety comment starting with `SAFETY:` and provide technical justification. | `lint.unsafety`                      | Include only safety annotations where performance is critical, e.g., under low-level system calls. |
| Doc comments are part of AST                | Ensure `#[doc]` attributes are present to ensure visibility to documentation tooling.                                                           | `lint::doc_comments_are_part_of_ast` | Use meaningful format strings in comments for better readability.                                  |
| Don't omit patterns on non_exhaustive enums | Prefer listing all enum variants in match statements over `non_exhaustive`.                                                                     | `lint.non_exhaustive_enums`          | Use `#[non_exhaustive]` only when the enum will change or has a clear path to stabilization.       |
| `const` functions responsibly               | Mark `const` functions only if you expect them to remain const, otherwise document breaking changes.                                            | `lint::must_use`                     | Use `const fn` for helper functions that are unlikely to be used elsewhere.                        |
| Don't run code before `main()`              | Avoid using compiler-generated crates like `ctor`.                                                                                              | `lint::prevents_panic_before_main()` | Ensure entry points call `main()` explicitly before any logic runs.                                |

## Code Examples (Examples)

### Type Inference Optimization

Use `#[must_use]` for types that are returned by functions that otherwise would be annotated `#[must_use]`.

Example:

```rust
fn get_val<T>() -> Option<T> {
    let result = map.get(&key);
    match result {
        Some(T::Value) => T::Value,
        None => None,
    }
}
```
````

### Accessor Patterns with Closures

```rust
// Read-only access
impl Foo {
    fn bar<R>(&self, f: impl FnOnce(&Bar) -> R) -> R {
        f(self.baz.clone())
    }
}

// Mutable access with closure
impl Foo {
    fn baz_with_closure<R>(f: impl FnMut(&mut Baz) -> R) -> R {
        let mut data = &self.baz;
        match f(data) {
            Ok(r) => self.baz.clone(),
            Err(_) => panic!("Invalid mutation"),
        }
    }
}
```

### `#[non_exhaustive]` Usage

Mark enums with public fields as `#[non_exhaustive]`. Use it only when:

- The enum will change in the future.
- Users can use functional record initialization syntax.
- The struct fields are unlikely to change (e.g., due to existing ontology).

```rust
// Marked public enums
#[non_exhaustive]
pub enum Status {
    PENDING,
    PUBLISHED,
}

// Can't construct with:
struct Example(pub(crate) pub_status);

fn process_example(status: Status) -> Self {
    status.clone() // No type inference needed!
}
```

### `#[must_use]` Usage

Use `#[must_use]` for return values of functions that would otherwise be annotated `#[must_use]`.

Examples:

```rust
fn get_val<T>() -> Option<T> {
    let result = map.get(&key);
    match result {
        Some(T::Value) => T::Value,
        None => None,
    }
}

// Marked types that can be used in other contexts (e.g., error types or lazily evaluated computations)
pub struct Result<T> where {}
pub enum StatusOrError {
    Ok(Some(T)),
    Err(Error),
}

```

### Consistent Code Patterns

- Use `write!()` and `format!()` for concatenation and substitution, not `+` for multiple strings.
- Prefer `map` to `.into_iter().collect()`.
- Avoid iterator combinator chains if direct iteration is clearer (e.g., in loops).

### Context Strings in Assertions

Always consider adding context:

```rust
// Better than: assert!(x > 0, "`x` is positive");
assert!(refund > 0, "Refund must be positive, got {:?}: {}", refund, refund.to_string());
```

### Prefer Pattern Matching Over Check-Then-Unwrap

Use match statements instead of pattern matching for guaranteed correctness.

Example:

```rust
// Matched correctly in type system
if let Some(value) = option {
    use_value(value);
}
```

Example (unsafe):

```rust
let _ = unsafe { option.unwrap() }; // Not allowed in production code
```

### Use `expect` Over `unwrap` in Error Cases

Use `expect!` for panic prevention:

```rust
fn get_val<T>() -> Result<T> {
    let result = map.get(&key);
    expect_err(|| return Err(Error));
}
```

## Project Structure and Linting

- Ensure no lint errors across all code.
- Apply `#[non_exhaustive]` to enums/structs with public fields that are part of a public API used by other teams.
- Mark `const` functions responsibly if they may change or can help clients migrate.
- Avoid using unstable features.

## Use the Right Test Framework

Use the framework appropriate for your use case

---

This layout will ensure that the codebase follows best practices, is clear and maintainable, and passes all relevant linting checks.
