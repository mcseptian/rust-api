# Rust Style Guide Reference

## 1. General & API Design

- **Readability:** Favor clear, explicit code over overly clever logic.
- **Formatting:** Assume standard `rustfmt` rules apply automatically.
- **Booleans:** Avoid raw `bool` flags in public functions; use explicit `enum` types.
- **Flexible Parameters:** Accept `&str` over `&String`, `&[T]` over `&Vec<T>`, and `Into<T>` for generic conversions.

## 2. Imports & Modules

- **File Naming:** Name leaf modules after their primary type (e.g., `mod foo` containing `struct Foo` in `foo.rs`).
- **Non-Leaf Modules:** Non-leaf files must only declare submodules (`mod bar;`) and re-export items (`pub use bar::*;`).
- **No Glob Imports:** Avoid `use module::*` (exceptions: `use super::*` in tests, `use Enum::*` inside enum methods).
- **Import Scoping:**
  - Qualify top-level functions with their module name (`std::mem::transmute`).
  - Import types/traits directly into scope and use them unqualified (`Regex`).
  - Keep imports at module-level granularity (avoid nested `{a::{b, c}}`).

## 3. Ownership & Borrowing

- **Smart Pointers:** Avoid `Rc` / `Arc` unless shared ownership is explicitly required.
- **Borrows vs Clones:** Prefer borrowing (`&T` / `&mut T`) over calling `.clone()` or copying.

## 4. Types & Safety

- **Newtypes:** Wrap primitives (`struct UserId(u64)`) for domain safety.
- **Named Structs:** Prefer named fields over tuple structs for public data structures.
- **Derives:** Prefer standard derives (`Debug`, `Clone`, `Default`, `PartialEq`, `From`) over custom methods.
- **Error Handling:** Use `.expect("reason")` instead of raw `.unwrap()`.
- **Pattern Matching:** Prefer `if let` / `match` over check-then-unwrap.
- **Unsafe Code:** Minimize `unsafe` scope and mandate a `// SAFETY:` explanatory comment above every block.

## 5. Project-Specific Conventions

- **Application Type:** Actix-web REST API with SQLite/Diesel.
- **Modules:** Keep `main.rs` focused on startup and routing; keep handlers in `routes.rs`.
- **Imports:** Do not use glob imports except where already established by Diesel macros.
- **Errors:** Route handlers should return meaningful HTTP errors instead of panicking.
- **Tasks:** Task status must be one of `to do`, `in progress`, `done`, or `blocked`.
- **Timestamps:** Store task `created_at` and `updated_at` as ISO 8601 strings.
- **Routes:** Preserve the documented route set: `/`, `/users`, `/get_users`, `/tasks`, `/get_tasks`, and `/update_db`.
