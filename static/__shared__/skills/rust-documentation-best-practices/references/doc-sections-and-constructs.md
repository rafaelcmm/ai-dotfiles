# Standard Doc Sections and Construct Walkthroughs

Worked examples for the standard rustdoc section headings (`# Examples`,
`# Panics`, `# Errors`, `# Safety`, `# Complexity`) and for documenting
structs, enums, and traits.

## Contents

- [Standard Section Headings](#standard-section-headings)
  - [`# Examples` — Required where usage is not obvious](#-examples-required-where-usage-is-not-obvious)
  - [`# Panics` — Document every panic condition](#-panics-document-every-panic-condition)
  - [`# Errors` — Document all error variants returned](#-errors-document-all-error-variants-returned)
  - [`# Safety` — Mandatory for every `unsafe` function](#-safety-mandatory-for-every-unsafe-function)
  - [`# Complexity` — For data structures and algorithms](#-complexity-for-data-structures-and-algorithms)
- [Documenting Structs and Fields](#documenting-structs-and-fields)
- [Documenting Enums](#documenting-enums)
- [Documenting Traits](#documenting-traits)

## Standard Section Headings

rustdoc recognizes these heading names and renders them prominently.
Always use them in this order when applicable:

### `# Examples` — Required where usage is not obvious

Required for primary entry points, and for any item whose correct use is not
obvious from the signature alone. A doctest that only restates an obvious call is
a compiled, maintained artifact carrying no information the summary line did not
already give — prefer one example that shows a real decision over three that show
the signature again. `# Safety`, `# Panics` and `# Errors` are contract rather
than illustration and are never optional on the items that need them.

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

