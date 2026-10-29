# Documenting Go Constructs

Worked examples for doc comments on packages, functions, methods, types, and
runnable Example functions.

## Contents

- [Comment Types](#comment-types)
  - [Doc Comments — `//` preceding declarations](#doc-comments-preceding-declarations)
  - [Inline Comments — `//` within function bodies](#inline-comments-within-function-bodies)
  - [Block Comments — `/* ... */`](#block-comments-)
- [Package Documentation](#package-documentation)
  - [Simple package comment](#simple-package-comment)
  - [Multi-paragraph package comment](#multi-paragraph-package-comment)
  - [doc.go pattern — for large packages](#docgo-pattern-for-large-packages)
- [Function and Method Documentation](#function-and-method-documentation)
  - [Core conventions](#core-conventions)
  - [Document parameters when their meaning isn't obvious](#document-parameters-when-their-meaning-isnt-obvious)
  - [Document return values](#document-return-values)
  - [Document panics explicitly](#document-panics-explicitly)
- [Type Documentation](#type-documentation)
  - [Structs — document the type and each exported field](#structs-document-the-type-and-each-exported-field)
  - [Interfaces](#interfaces)
  - [Constants and variables](#constants-and-variables)
  - [Group-documented constants](#group-documented-constants)
- [Example Functions](#example-functions)

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

