Refactor this Rust file with full adherence to Rust idioms, safety, and best practices.

### Module & Imports
- Place the file in the correct module/submodule; merge or split modules if clarity improves.
- Remove unused imports; tighten visibility with `pub`, `pub(crate)`, or `pub(super)` as appropriate.
- Clean up re-exports.

### Code Quality
- Eliminate duplicate logic and dead code.
- Replace non-idiomatic patterns:
  - Prefer enums over booleans.
  - Use iterators instead of manual loops.
  - Use `?` instead of `unwrap`.
  - Reduce unnecessary `clone`s.
  - Tighten lifetimes.

### Naming & Types
- Follow Rust naming conventions (`snake_case`, `PascalCase`).
- Use descriptive, concise identifiers.
- Strengthen type safety:
  - Introduce newtypes/enums/structs where appropriate.
  - Avoid nested `Option<Option<T>>`.
  - Prefer `&str` over `String` when possible.

### Error Handling
- Replace panics with proper error handling unless unrecoverable.
- Use `thiserror` or custom error types.

### Documentation
- Add Rustdoc for all public items:
  - Explain purpose, contracts, and caveats.
- Improve inline comments for clarity.

### Testing
- Add/update tests in `tests/` or `#[cfg(test)]`:
  - Cover positive, negative, and edge cases.

### Performance
- Remove needless allocations and clones.
- Replace expensive patterns with efficient iterators.

### Constraints
- Do not introduce breaking changes unless fixing incorrect behavior.
- Delete the file if obsolete.
- Use external crates where appropriate.
