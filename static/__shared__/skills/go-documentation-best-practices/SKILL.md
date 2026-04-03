---
name: go-documentation-best-practices
description: >
  Comprehensive guide for documenting Go code with godoc-compatible comments,
  package documentation, and inline commentary. Use this skill whenever writing,
  reviewing, or improving Go documentation — including package comments, function
  and method doc comments, type documentation, struct field comments, example
  functions, doc.go files, deprecation notices, and inline comments. Trigger on
  any .go file that needs documentation added, reviewed, or improved — including
  libraries, CLI tools, HTTP servers, gRPC services, and any Go package or module.
---

# Go Documentation Best Practices

## Philosophy

Go documentation is intentionally plain and minimal. The godoc tool parses ordinary
comments — no special syntax, no markup language, no annotations. The goal is that
comments read naturally as prose *even without* godoc rendering them.

**Core rules:**
1. Every exported identifier (type, function, method, constant, variable) MUST have a doc comment.
2. Doc comments are full sentences that **begin with the name** of the thing being described.
3. Comments end with a period.
4. No blank line between the comment and the declaration.
5. Explain *what* and *when* — not *how*. The code shows how.

---

## Comment Types

### Doc Comments — `//` preceding declarations

Used for all exported identifiers. Parsed and displayed by godoc and pkg.go.dev.

```go
// Package ratelimit provides sliding-window rate limiting for HTTP services.
//
// It supports per-IP, per-user, and global rate limits with configurable
// windows and burst capacity. Limits are stored in Redis for distributed use.
//
// Basic usage:
//
//	limiter := ratelimit.New(ratelimit.Config{
//	    Rate:  100,
//	    Burst: 10,
//	    Window: time.Minute,
//	})
//	http.Handle("/api/", limiter.Middleware(apiHandler))
package ratelimit
```

### Inline Comments — `//` within function bodies

Used to explain non-obvious logic, workarounds, and intent. Never placed inside doc comments.
These are NOT parsed by godoc.

```go
// Must release the lock before calling notifyWaiters — notifyWaiters acquires
// its own lock and will deadlock if we hold this one.
mu.Unlock()
notifyWaiters()
```

### Block Comments — `/* ... */`

Rarely used in Go. Acceptable for large copyright headers or temporarily disabling code.
**Never use for doc comments** — godoc only parses `//` style comments.

```go
/*
 * Copyright 2024 Acme Corp. All rights reserved.
 * Licensed under the Apache License, Version 2.0.
 */
```

---

## Package Documentation

Every package must have a package comment. For simple packages, one line is fine.
For complex packages, use a `doc.go` file.

### Simple package comment

```go
// Package uuid generates and validates RFC 4122 universally unique identifiers.
package uuid
```

### Multi-paragraph package comment

```go
// Package httputil provides utilities for building and testing HTTP servers.
//
// It includes middleware for logging, authentication, rate limiting, and
// CORS. All middleware is composable and designed for use with the standard
// net/http package.
//
// # Middleware
//
// Middleware functions follow the standard pattern of wrapping an http.Handler:
//
//	chain := httputil.Chain(
//	    httputil.Logger(os.Stdout),
//	    httputil.Auth(authProvider),
//	    httputil.CORS(corsConfig),
//	)
//	http.ListenAndServe(":8080", chain.Then(myHandler))
//
// # Error Handling
//
// All middleware writes structured errors using [WriteJSONError]. Errors are
// logged with request IDs for correlation with the access log.
//
// [WriteJSONError]: https://pkg.go.dev/example.com/httputil#WriteJSONError
package httputil
```

### doc.go pattern — for large packages

```go
// doc.go — package-level documentation lives here

// Package storage implements a pluggable storage backend abstraction.
//
// # Backends
//
// Three backends are provided out of the box:
//   - [memory.Backend]: in-process storage, suitable for testing.
//   - [postgres.Backend]: PostgreSQL via pgx, with connection pooling.
//   - [s3.Backend]: AWS S3 for object storage.
//
// # Implementing a Custom Backend
//
// Implement the [Backend] interface:
//
//	type Backend interface {
//	    Get(ctx context.Context, key string) ([]byte, error)
//	    Put(ctx context.Context, key string, value []byte) error
//	    Delete(ctx context.Context, key string) error
//	}
//
// See [memory.Backend] for a reference implementation.
package storage
```

---

## Function and Method Documentation

### Core conventions

Comments **must begin with the function name** (or for methods, `MethodName`):

```go
// Encode writes the JSON encoding of v to w.
//
// Encoding is idempotent: calling Encode with the same value produces
// the same output. Circular references in v cause an infinite loop.
//
// The w is flushed after encoding completes.
func Encode(w io.Writer, v any) error { ... }

// ParseURL parses rawURL into a URL structure.
//
// It is assumed that rawURL was received in an HTTP request, so the
// URL is assumed to be in absolute form or an absolute path.
func ParseURL(rawURL string) (*URL, error) { ... }
```

### Document parameters when their meaning isn't obvious

```go
// Retry calls fn up to maxAttempts times, waiting backoff duration between
// attempts. It returns the first nil error returned by fn, or the last
// non-nil error if all attempts fail.
//
// backoff is the fixed wait time between retries. For exponential backoff,
// use [RetryExponential] instead.
//
// If ctx is cancelled before fn succeeds, Retry returns ctx.Err().
func Retry(ctx context.Context, maxAttempts int, backoff time.Duration, fn func() error) error { ... }
```

### Document return values

```go
// NewClient creates a new API client using the given config.
//
// NewClient validates the config and establishes an initial connection to
// verify credentials. It returns an error if the config is invalid or if
// the connection cannot be established within config.Timeout.
func NewClient(cfg Config) (*Client, error) { ... }
```

### Document panics explicitly

```go
// MustCompile compiles the given pattern and returns the resulting Regexp.
// It panics if the pattern cannot be compiled.
// It is intended for use in global variable initializations.
func MustCompile(pattern string) *Regexp { ... }
```

---

## Type Documentation

### Structs — document the type and each exported field

```go
// Request represents an HTTP request to be sent by a Client.
//
// A Request may be reused across multiple Client.Do calls, but callers
// must not mutate it between calls. Use Clone to create an independent copy.
type Request struct {
    // Method specifies the HTTP method (GET, POST, PUT, etc.).
    // For client requests, an empty string means GET.
    Method string

    // URL specifies the URL to access.
    // For client requests, the URL's Host specifies the server to
    // connect to, and the URL's Path must be an absolute path.
    URL *url.URL

    // Header contains the request header fields.
    //
    // If a server receives a request with header lines:
    //
    //	Host: example.com
    //	accept-encoding: gzip, deflate
    //
    // then Header = map[string][]string{
    //     "Accept-Encoding": {"gzip, deflate"},
    // }
    Header http.Header

    // Body is the request's body, if any.
    // A nil body means the request has no body, such as a GET request.
    // The Client's Transport is responsible for calling Close.
    Body io.ReadCloser

    // ContentLength records the length of the associated content.
    // -1 means length is unknown. Values >= 0 mean at most that many
    // bytes may be read from Body.
    ContentLength int64
}
```

### Interfaces

```go
// Writer is the interface that wraps the basic Write method.
//
// Write writes len(p) bytes from p to the underlying data stream.
// It returns the number of bytes written from p (0 <= n <= len(p)) and
// any error encountered that caused the write to stop early.
//
// Write must return a non-nil error if it returns n < len(p).
// Write must not modify the slice data, even temporarily.
//
// Implementations must not retain p.
type Writer interface {
    Write(p []byte) (n int, err error)
}
```

### Constants and variables

```go
// Common HTTP methods. These are defined in RFC 9110.
const (
    MethodGet     = "GET"
    MethodHead    = "HEAD"
    MethodPost    = "POST"
    MethodPut     = "PUT"
    MethodDelete  = "DELETE"
    MethodPatch   = "PATCH"
    MethodOptions = "OPTIONS"
)

// ErrNotFound is returned when a requested resource does not exist.
// Callers should use errors.Is(err, ErrNotFound) to check for this error.
var ErrNotFound = errors.New("not found")
```

### Group-documented constants

```go
// HTTP response status codes as registered with IANA.
// See: https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml
const (
    StatusOK                   = 200
    StatusCreated              = 201
    StatusNoContent            = 204
    StatusMovedPermanently     = 301
    StatusNotFound             = 404
    StatusInternalServerError  = 500
)
```

---

## Example Functions

Example functions are compiled and run as tests via `go test`. They appear in godoc.
Place in `_test.go` files with the naming convention `ExampleXxx` or `ExampleXxx_suffix`.

```go
// example_test.go

// ExampleClient_Do demonstrates making an authenticated API request.
func ExampleClient_Do() {
    client := api.NewClient(api.Config{
        BaseURL: "https://api.example.com",
        APIKey:  "your-api-key",
    })

    resp, err := client.Do(context.Background(), &api.Request{
        Method: "GET",
        Path:   "/users/123",
    })
    if err != nil {
        log.Fatal(err)
    }
    defer resp.Body.Close()

    fmt.Println(resp.StatusCode)
    // Output: 200
}

// ExampleEncode shows how to encode a value to JSON with pretty-printing.
func ExampleEncode() {
    type Point struct{ X, Y int }
    buf := &bytes.Buffer{}
    if err := json.NewEncoder(buf).Encode(Point{1, 2}); err != nil {
        log.Fatal(err)
    }
    fmt.Print(buf.String())
    // Output: {"X":1,"Y":2}
}
```

---

## Godoc Formatting Rules

Godoc uses a simple markdown-like format (since Go 1.19):

### Paragraphs
Blank comment lines create paragraph breaks:

```go
// ParseConfig reads a configuration file and returns a Config.
//
// The file must be in TOML format. Environment variables are expanded
// in string values using os.Expand.
//
// ParseConfig returns an error if the file does not exist, cannot be
// parsed, or contains unknown fields.
func ParseConfig(path string) (*Config, error) { ... }
```

### Headings (Go 1.19+)
Lines starting with `#` become section headings:

```go
// Package crypt implements multiple hashing algorithms.
//
// # Supported Algorithms
//
// The following algorithms are supported:
//   - bcrypt (recommended for passwords)
//   - argon2id (recommended for modern systems)
//   - sha256, sha512 (for non-password data)
//
// # Security Notes
//
// Never use MD5 or SHA1 for new code. Both are cryptographically broken.
package crypt
```

### Code blocks
Indented text renders as a code block:

```go
// Dial connects to the server at addr using the specified protocol.
//
// Usage:
//
//	conn, err := net.Dial("tcp", "golang.org:80")
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer conn.Close()
func Dial(network, addr string) (Conn, error) { ... }
```

### Lists
Bullet and numbered lists:

```go
// New returns a new rate limiter with the given config.
//
// The config must satisfy:
//   - Rate must be positive
//   - Burst must be >= Rate
//   - Window must be at least 1 second
//
// Invalid configs cause a panic. Use [Validate] to check before calling New.
func New(cfg Config) *Limiter { ... }
```

### Links
Cross-references use `[Name]` syntax (links to same package) or URLs:

```go
// Clone returns a deep copy of r. See [Request.Clone] for caveats.
//
// For the URL specification, see [RFC 3986].
//
// [RFC 3986]: https://www.rfc-editor.org/rfc/rfc3986
func Clone(r *Request) *Request { ... }
```

---

## Deprecation

```go
// Deprecated: Use [NewClientV2] instead. NewClient will be removed in v3.0.
//
// NewClient creates a client using the v1 API. The v1 API was deprecated in
// favor of v2 which supports TLS 1.3 and HTTP/2.
func NewClient(addr string) *Client { ... }
```

---

## Known Bugs

Top-level comments beginning with `BUG(who)` appear in the "Bugs" section of godoc:

```go
// BUG(rsc): The Title function does not handle Unicode punctuation correctly.
// Tracked in https://github.com/golang/go/issues/12345
```

---

## Inline Comments Best Practices

### Explain WHY, not WHAT

```go
// ✅ Good — explains the reason for a non-obvious choice
// Use sync.Map instead of a mutex-protected map here because this map
// is read frequently and written rarely (classic read-heavy workload).
var cache sync.Map

// ✅ Good — explains a gotcha
// http.FileServer strips the prefix before serving, so we must also
// strip it when constructing the URL for client-side redirects.
prefix := "/static/"
handler := http.StripPrefix(prefix, http.FileServer(http.Dir("static")))

// ❌ Bad — restates what the code already says
i++ // increment i
mu.Lock() // lock the mutex
```

### Annotate workarounds and external constraints

```go
// Workaround for https://github.com/golang/go/issues/XXXX:
// context.WithTimeout leaks a goroutine if the parent context is never
// cancelled. We work around this by always calling cancel() immediately
// after use, even on success.
ctx, cancel := context.WithTimeout(parent, 5*time.Second)
defer cancel()
```

### TODO/FIXME/HACK format

```go
// TODO(alice): Migrate to the v2 API once it supports streaming (ETA Q3 2024).
// FIXME: This is O(n²) — replace with a hash join when table sizes grow.
// HACK: The provider returns status 200 for errors; check the body for "error".
// NOTE: This function must be called from the same goroutine as Start.
```

---

## Anti-Patterns

```go
// ❌ Comment doesn't begin with the function name
// Creates a new user in the database.
func NewUser(name string) *User { ... }

// ✅ Correct
// NewUser creates a new user in the database with the given name.
func NewUser(name string) *User { ... }

// ❌ No period at end
// Process handles incoming messages
func Process(msg Message) { ... }

// ✅ Correct
// Process handles incoming messages from the queue.
func Process(msg Message) { ... }

// ❌ Blank line between comment and declaration (breaks godoc association)
// Connect establishes a connection to the server.

func Connect(addr string) error { ... }

// ❌ Redundant comment
// i is the loop counter
for i := 0; i < n; i++ { ... }

// ❌ Exported function without doc comment
func HandleError(err error) { ... }  // godoc will warn about this
```

---

## godoc CLI and Tools

```bash
# Serve godoc locally for your module
godoc -http=:6060

# View package docs in terminal
go doc ./...
go doc net/http.Request
go doc net/http.Request.Header

# Check for missing doc comments (go vet)
go vet ./...

# staticcheck also checks documentation
staticcheck ./...

# Generate documentation as static HTML
godoc -http=:8080 &
wget -r -np -N -E -p -k http://localhost:8080/pkg/github.com/yourname/yourpkg/
```

---

## Documentation Checklist

- [ ] Every exported type, function, method, constant, and variable has a doc comment
- [ ] Package has a package comment (or `doc.go` for large packages)
- [ ] Doc comments begin with the name of the thing they describe
- [ ] Doc comments are full sentences ending with a period
- [ ] No blank line between comment and declaration
- [ ] Parameters/return values explained when their meaning isn't obvious from the name
- [ ] Panics documented explicitly with `panics if...`
- [ ] Error conditions documented for functions returning `error`
- [ ] Deprecations use the `Deprecated:` prefix
- [ ] Examples use `Example` functions in `_test.go` files with `// Output:` assertions
- [ ] Cross-references use `[SymbolName]` or `[pkg.SymbolName]` syntax (Go 1.19+)
- [ ] Inline comments explain WHY, not WHAT
- [ ] TODO/FIXME include owner and context
- [ ] No `//` comment on the same line as code (put it on the line above)
- [ ] `BUG(who):` comments for known issues with reproduction info
