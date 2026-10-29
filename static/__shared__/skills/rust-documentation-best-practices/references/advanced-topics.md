# Advanced Rustdoc Topics

Doctest code annotations, intra-doc links, `#[doc]` attributes, crate-level
lints, and the `cargo test`/`cargo doc` commands for running and checking docs.

## Contents

- [Doctest Code Annotations](#doctest-code-annotations)
- [Intra-doc Links](#intra-doc-links)
- [`#[doc]` Attributes](#doc-attributes)
- [Crate-Level Lints](#crate-level-lints)
- [Running Doctests](#running-doctests)

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

