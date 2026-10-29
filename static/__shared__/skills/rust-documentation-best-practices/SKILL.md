---
name: rust-documentation-best-practices
description: >
  Comprehensive guide for documenting Rust code using rustdoc, doc comments, doctests,
  and structured section headings. Use this skill whenever writing, reviewing, or improving
  Rust documentation — including /// and //! comments, rustdoc section headings (Examples,
  Panics, Errors, Safety, Complexity), doc attributes, intra-doc links, doctests,
  module documentation, crate-level docs, and inline comments. Trigger on any .rs file
  that needs documentation added, reviewed, or improved — including libraries, binaries,
  proc macros, async code, unsafe blocks, and any Rust crate.
---

# Rust Documentation Best Practices

## Philosophy

Rust documentation is built into the language and enforced by the ecosystem.
`cargo doc` generates beautiful, linked documentation from `///` comments.
Doctests (code examples in doc comments) are **compiled and run as tests** —
this makes documentation executable and always up-to-date.

**Core rules:**
1. Every `pub` item must have a doc comment (enforce with `#![warn(missing_docs)]`).
2. Inner doc comments (`//!`) document the enclosing item (crate, module).
3. Outer doc comments (`///`) document the following item.
4. Use Markdown inside doc comments — it renders fully in rustdoc output.
5. Structure docs with standard section headings: `# Examples`, `# Panics`, `# Errors`, `# Safety`.
6. Code examples in ` ```rust ``` ` blocks are compiled and run as tests by default.

---

## Scope and restraint

Canonical policy: `commenting-standards`. This section applies it to rustdoc.

### Coverage is absolute, length is proportional

Every `pub` item keeps its doc comment, and `#![warn(missing_docs)]` stays on —
the Core rules above are not relaxed. What scales is depth, never presence.
Depth is owed in proportion to visibility, subtlety, and how far a mistake
travels.

A self-evident signature is fully documented by one assertive sentence:

```rust
/// Returns the number of queued jobs.
pub fn len(&self) -> usize
```

That is finished documentation. Rust's types carry a great deal — `Result`,
`Option`, ownership and lifetimes are all visible in the signature — so prose
that restates them is pure padding, and padding is a defect.

### Contract sections stay; `# Examples` is proportional

`# Safety` is **mandatory for every `unsafe` fn** — never omit or shorten it away.
`# Panics` and `# Errors` are mandatory wherever the item can panic or return
`Err`. These are contract, not illustration, and a caller cannot recover the
information from anywhere else. Brevity applies to their content, never to their
presence.

`# Examples` is different in kind. It is required for primary entry points and
for any item whose correct use is not obvious from the signature alone. A doctest
that only restates an obvious call adds a compiled, maintained artifact and no
information — the summary line already carried that fact. Prefer one example that
shows a real decision (an error path, a non-obvious combination, a lifetime that
needs explaining) over three that show the signature again.

### Document the present

A doc comment states what the item does now; it is not a change log. `git log`,
`git blame` and the pull request own history and keep it accurate for free. Never
write provenance (`Extracted from the client module when...`), removed behaviour
(`This used to take a &str`), dated claims, supersession notices, or ticket
archaeology. Such a comment goes wrong at the next edit, and it is the one kind
of staleness a reader cannot detect, because nothing in the crate contradicts it.

Legitimately time-bearing, because each is present and actionable: a
`TODO`/`FIXME` with a tracking reference; a workaround with its upstream issue
and the condition that retires it; `#[deprecated(note = "use X instead")]`, which
is the language's own mechanism; and an invariant with its expiry trigger.

### Assertive voice and the stop rule

Lead with one declarative sentence in the third person indicative — `Parses a
duration`, not `This function will parse`. Cut hedges and self-reference. A doc
comment is finished when a competent caller can use the item correctly without
reading its body; implementation reasoning goes in a `//` comment at the line it
explains, and history goes nowhere.

### Before and after

```rust
/// This function is basically a helper which is used in order to parse a port.
///
/// It is important to note that it returns a Result. Originally it lived in
/// `config::legacy`, but it moved here during the Q3 refactor, and previously
/// it accepted port 0.
///
/// # Examples
///
/// ```
/// let p = mylib::parse_port("8080").unwrap();
/// assert_eq!(p, 8080);
/// ```
pub fn parse_port(s: &str) -> Result<u16, ParseError>
```

```rust
/// Parses a TCP port, rejecting `0`.
///
/// Port 0 asks the OS to assign an arbitrary free port. That is valid in a
/// `bind` call but never valid in configuration, where it silently produces a
/// service reachable on an address nobody can predict.
///
/// # Errors
///
/// Returns [`ParseError::NotANumber`] for non-numeric input,
/// [`ParseError::OutOfRange`] above `65535`, and [`ParseError::Reserved`] for `0`.
pub fn parse_port(s: &str) -> Result<u16, ParseError>
```

Shorter because the padding left, not because information did. The obvious
doctest went; `# Errors` arrived, because a `Result` in the signature tells a
caller that it can fail but not how.

---

## Comment Types

### `///` — Outer Doc Comments (document the item that follows)

Used for: functions, structs, enums, traits, type aliases, constants, fields.

```rust
/// Parses a string slice into a `Duration`.
///
/// Accepts values in the format `"<number><unit>"` where unit is one of:
/// `ns` (nanoseconds), `us` (microseconds), `ms` (milliseconds), `s` (seconds),
/// `m` (minutes), `h` (hours), or `d` (days).
///
/// # Examples
///
/// ```
/// use mylib::parse_duration;
///
/// let d = parse_duration("500ms").unwrap();
/// assert_eq!(d.as_millis(), 500);
/// ```
///
/// # Errors
///
/// Returns `Err(ParseError::InvalidFormat)` if the string does not match
/// the expected format.
/// Returns `Err(ParseError::Overflow)` if the value exceeds `u64::MAX` nanoseconds.
///
/// # Panics
///
/// Does not panic. (Note: explicitly state this if no panics can occur.)
pub fn parse_duration(s: &str) -> Result<Duration, ParseError> { ... }
```

### `//!` — Inner Doc Comments (document the enclosing item)

Used for: crate root (`lib.rs`, `main.rs`), module files.
Must appear at the **top of the file**, before any items.

```rust
//! # mylib — High-performance JSON streaming parser
//!
//! `mylib` provides a zero-copy, streaming JSON parser designed for
//! processing large JSON files with minimal memory allocation.
//!
//! ## Quick Start
//!
//! ```rust
//! use mylib::Parser;
//!
//! let parser = Parser::new();
//! for event in parser.parse(json_bytes) {
//!     println!("{:?}", event?);
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `serde`: Enables deserialization into any `Deserialize` type.
//! - `async`: Enables async streaming via `tokio::io::AsyncRead`.
//!
//! ## Design
//!
//! The parser uses a state machine approach with no heap allocation in the
//! hot path. See [`Parser`] and [`Event`] for the main API surface.

pub mod parser;
pub mod error;
```

---


For worked examples of each standard section (`# Examples`, `# Panics`,
`# Errors`, `# Safety`, `# Complexity`) and of documenting structs, enums, and
traits, read [references/doc-sections-and-constructs.md](references/doc-sections-and-constructs.md).

## Module Documentation

```rust
//! # Parser module
//!
//! Provides the core JSON parsing state machine.
//!
//! ## Architecture
//!
//! The parser is split into three layers:
//! 1. **Lexer** ([`lexer`]) — tokenizes raw bytes into [`Token`]s.
//! 2. **Parser** ([`Parser`]) — validates token sequences and emits [`Event`]s.
//! 3. **Visitor** ([`Visitor`]) — provides a callback-based API over events.
//!
//! For most use cases, use [`Parser`] directly.
//!
//! ## Example
//!
//! ```
//! use mylib::parser::Parser;
//!
//! let json = br#"{"name": "Alice", "age": 30}"#;
//! let events: Vec<_> = Parser::new().parse(json).collect::<Result<_, _>>()?;
//! # Ok::<(), mylib::Error>(())
//! ```

use crate::lexer::Lexer;
```

---

## Inline Comments (Non-Doc)

```rust
// ✅ Explain WHY — the code shows what, the comment explains why
// We use `Ordering::AcqRel` instead of `SeqCst` because we only need
// visibility of writes to this specific atomic, not a total order across
// all atomics in the program. This is 3x faster on ARM.
let _ = counter.fetch_add(1, Ordering::AcqRel);

// ✅ Explain non-obvious unsafe invariant
// SAFETY: We hold the write lock on `self.data`, so no other thread can
// read or write during this slice operation.
let slice = unsafe { self.data.get_unchecked_mut(start..end) };

// ✅ Document surprising external behavior
// reqwest silently follows redirects by default. Disable this to detect
// 301/302 responses explicitly — the OAuth flow needs to intercept them.
let client = Client::builder().redirect(Policy::none()).build()?;

// ❌ Avoid obvious comments
let len = vec.len(); // get the length
```

### SAFETY comments for unsafe blocks

Every `unsafe` block must have a `// SAFETY:` comment:

```rust
// SAFETY: `ptr` is guaranteed non-null by the API contract of `allocate()`,
// and we just allocated with the correct layout for `T`, so alignment and
// initialization are satisfied.
let reference = unsafe { &*ptr };
```

---


## Advanced Topics

Doctest code annotations, intra-doc links, `#[doc]` attributes, crate-level
lints, and running/checking doctests are in
[references/advanced-topics.md](references/advanced-topics.md).

## Documentation Checklist

- [ ] Every `pub` item has a `///` doc comment
- [ ] Crate root (`lib.rs`) has `//!` comment with overview, quickstart, and feature flags
- [ ] All modules have `//!` inner doc comments
- [ ] `# Examples` section present for all public functions, methods, and types
- [ ] `# Errors` section documents all `Err` variants returned
- [ ] `# Panics` section documents every panic condition (or states it doesn't panic)
- [ ] `# Safety` section present on every `unsafe fn` — explicit and thorough
- [ ] `# Complexity` section for algorithmic or data structure methods
- [ ] Doctests are real, runnable code with `assert_eq!` or similar validation
- [ ] Boilerplate hidden with `#` prefix lines so examples are clean
- [ ] `no_run` / `should_panic` / `compile_fail` annotations used where appropriate
- [ ] Intra-doc links use `[`Item`]` format for all referenced types
- [ ] Every `unsafe {}` block has a `// SAFETY:` comment explaining the invariants
- [ ] `#![warn(missing_docs)]` enabled in `lib.rs`
- [ ] `#![deny(rustdoc::broken_intra_doc_links)]` enabled
- [ ] Deprecated items use `#[deprecated(since = "...", note = "...")]`
- [ ] Struct fields are individually documented
- [ ] Enum variants are individually documented with their semantics
- [ ] Inline comments explain WHY — not WHAT
