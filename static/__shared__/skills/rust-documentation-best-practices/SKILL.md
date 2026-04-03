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

## Standard Section Headings

rustdoc recognizes these heading names and renders them prominently.
Always use them in this order when applicable:

### `# Examples` — Mandatory for all public items

Code examples are compiled and executed as tests. Write them as real, runnable code:

```rust
/// Splits a string by a delimiter, returning a `Vec<&str>`.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let parts = mylib::split("a,b,c", ',');
/// assert_eq!(parts, vec!["a", "b", "c"]);
/// ```
///
/// Empty input returns a vec with one empty string:
///
/// ```
/// let parts = mylib::split("", ',');
/// assert_eq!(parts, vec![""]);
/// ```
pub fn split<'a>(s: &'a str, delim: char) -> Vec<&'a str> { ... }
```

### `# Panics` — Document every panic condition

```rust
/// Returns the element at `index` in the buffer.
///
/// # Panics
///
/// Panics if `index >= self.len()`.
///
/// ```should_panic
/// let buf = Buffer::new(10);
/// buf.get(99); // panics: index out of bounds
/// ```
pub fn get(&self, index: usize) -> &T { ... }
```

### `# Errors` — Document all error variants returned

```rust
/// Reads the configuration file at `path`.
///
/// # Errors
///
/// - [`Error::Io`]: If the file cannot be opened or read.
/// - [`Error::Parse`]: If the file is not valid TOML.
/// - [`Error::Missing`]: If a required key is absent from the config.
///
/// # Examples
///
/// ```no_run
/// let config = Config::from_file("config.toml")?;
/// # Ok::<(), mylib::Error>(())
/// ```
pub fn from_file(path: &Path) -> Result<Config, Error> { ... }
```

### `# Safety` — Mandatory for every `unsafe` function

```rust
/// Returns a mutable reference to the value at the given memory address.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is non-null and properly aligned for type `T`.
/// - `ptr` points to initialized memory.
/// - The returned reference does not outlive the data pointed to by `ptr`.
/// - No other references to the same data exist while this reference is live
///   (no aliasing violations).
///
/// Violating any of these conditions results in **undefined behavior**.
///
/// # Examples
///
/// ```rust
/// let mut x: u32 = 42;
/// let ptr = &mut x as *mut u32;
/// let val = unsafe { deref_mut(ptr) };
/// *val = 100;
/// assert_eq!(x, 100);
/// ```
pub unsafe fn deref_mut<'a, T>(ptr: *mut T) -> &'a mut T {
    &mut *ptr
}
```

### `# Complexity` — For data structures and algorithms

```rust
/// Inserts a key-value pair into the map.
///
/// # Complexity
///
/// - Average: O(1) amortized
/// - Worst case: O(n) due to rehashing when the load factor threshold is exceeded
///
/// # Examples
///
/// ```
/// let mut map = HashMap::new();
/// map.insert("key", 42);
/// assert_eq!(map.get("key"), Some(&42));
/// ```
pub fn insert(&mut self, key: K, value: V) -> Option<V> { ... }
```

---

## Documenting Structs and Fields

```rust
/// A connection pool managing reusable TCP connections to a server.
///
/// Connections are created lazily on first use and returned to the pool
/// after each operation. Idle connections older than [`Pool::idle_timeout`]
/// are evicted by a background task.
///
/// # Examples
///
/// ```
/// let pool = Pool::builder()
///     .max_size(10)
///     .idle_timeout(Duration::from_secs(30))
///     .connect("localhost:5432")
///     .await?;
///
/// let conn = pool.get().await?;
/// conn.execute("SELECT 1").await?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Pool {
    /// Maximum number of connections the pool will create.
    /// Additional requests block until a connection is returned.
    pub max_size: usize,

    /// Duration after which an idle connection is closed and removed.
    /// Set to `Duration::MAX` to disable idle eviction.
    pub idle_timeout: Duration,

    /// Timeout for acquiring a connection from the pool.
    /// Returns `Err(PoolError::Timeout)` if exceeded.
    pub acquire_timeout: Duration,

    // Internal state — not documented (private)
    inner: Arc<PoolInner>,
}
```

---

## Documenting Enums

```rust
/// Represents errors that can occur during request processing.
#[derive(Debug)]
pub enum RequestError {
    /// The request body exceeded the configured size limit.
    ///
    /// Contains the actual size in bytes and the configured limit.
    BodyTooLarge { size: usize, limit: usize },

    /// The request could not be authenticated.
    ///
    /// This variant is returned when the `Authorization` header is absent,
    /// malformed, or contains an expired or revoked token.
    Unauthorized,

    /// An I/O error occurred while reading the request body.
    Io(#[from] std::io::Error),

    /// A required header is missing from the request.
    ///
    /// The inner `String` contains the header name.
    MissingHeader(String),
}
```

---

## Documenting Traits

```rust
/// A type that can be serialized to a sequence of bytes.
///
/// Implementors must ensure that serialization is deterministic:
/// the same value always produces the same byte sequence.
///
/// # Examples
///
/// Implementing `Serialize` for a custom type:
///
/// ```
/// struct Point { x: f64, y: f64 }
///
/// impl Serialize for Point {
///     fn serialize(&self) -> Vec<u8> {
///         let mut buf = Vec::with_capacity(16);
///         buf.extend_from_slice(&self.x.to_le_bytes());
///         buf.extend_from_slice(&self.y.to_le_bytes());
///         buf
///     }
/// }
///
/// let p = Point { x: 1.0, y: 2.0 };
/// assert_eq!(p.serialize().len(), 16);
/// ```
pub trait Serialize {
    /// Serializes `self` into a byte vector.
    ///
    /// # Panics
    ///
    /// Implementations should not panic. Return an appropriate error
    /// type using `SerializeWithError` for fallible serialization.
    fn serialize(&self) -> Vec<u8>;

    /// Returns the serialized size in bytes without allocating.
    ///
    /// The default implementation serializes and measures the result.
    /// Override for better performance when size can be computed cheaply.
    fn serialized_size(&self) -> usize {
        self.serialize().len()
    }
}
```

---

## Doctest Code Annotations

Control how rustdoc compiles and runs code blocks:

```rust
/// # Examples
///
/// This runs as a test (default):
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
///
/// This compiles but does NOT run (for examples that touch files, network, etc.):
/// ```no_run
/// std::fs::write("/tmp/example.txt", b"hello")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// This is expected to panic:
/// ```should_panic
/// panic!("this is expected");
/// ```
///
/// This verifies that the code fails to compile:
/// ```compile_fail
/// let x: i32 = "not an integer"; // type error
/// ```
///
/// Hide boilerplate with `#` (lines starting with `#` don't appear in docs
/// but are still compiled and run):
/// ```
/// # use std::collections::HashMap;
/// # let mut map = HashMap::new();
/// map.insert("key", 42);
/// assert_eq!(map["key"], 42);
/// ```
```

---

## Intra-doc Links

Link to other items using `[`backtick name`]` syntax:

```rust
/// Creates a new [`HashMap`] from the given iterator of key-value pairs.
///
/// This is equivalent to calling [`HashMap::new`] and then
/// [`HashMap::extend`]. See also [`HashMap::from_iter`].
///
/// For an ordered map, use [`BTreeMap`] instead.
///
/// [`HashMap`]: std::collections::HashMap
/// [`BTreeMap`]: std::collections::BTreeMap
pub fn collect_map<K, V>(iter: impl Iterator<Item = (K, V)>) -> HashMap<K, V> { ... }
```

Short-form intra-doc links (resolved automatically within same crate):

```rust
/// See [`Error`] for all error variants.
/// Calls [`Self::connect`] internally on first use.
/// Implements [`std::fmt::Display`].
```

---

## `#[doc]` Attributes

```rust
// Inline docs from a re-export into the current module
#[doc(inline)]
pub use crate::internal::RawBuffer;

// Hide an item from generated docs (e.g., macro internals)
#[doc(hidden)]
pub fn __internal_helper() { ... }

// Add a "deprecated" badge in docs
#[deprecated(since = "2.0.0", note = "Use `new_function` instead.")]
pub fn old_function() { ... }

// Document feature flags in lib.rs
#![doc(html_root_url = "https://docs.rs/mylib/1.0.0")]
```

---

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

## Crate-Level Lints

Add to `lib.rs` or `main.rs` to enforce documentation standards:

```rust
#![warn(missing_docs)]           // warn on any pub item without doc comment
#![warn(missing_debug_implementations)] // warn if pub types don't impl Debug
#![deny(rustdoc::broken_intra_doc_links)] // error on broken intra-doc links
#![warn(rustdoc::missing_crate_level_docs)] // warn if crate has no //! docs
```

---

## Running Doctests

```bash
# Run all doctests
cargo test --doc

# Run doctests for a specific module
cargo test --doc mylib::parser

# Generate and open documentation
cargo doc --open

# Include private items in docs
cargo doc --document-private-items

# Check for broken links and missing docs
cargo doc 2>&1 | grep warning
```

---

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
