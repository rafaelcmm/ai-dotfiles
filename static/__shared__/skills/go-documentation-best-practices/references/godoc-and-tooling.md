# Godoc Formatting and Tooling

Godoc comment formatting rules (paragraphs, headings, code blocks, lists,
links), deprecation and known-bugs conventions, and the `go doc`/`godoc`
commands for viewing and checking documentation.

## Contents

- [Godoc Formatting Rules](#godoc-formatting-rules)
  - [Paragraphs](#paragraphs)
  - [Headings (Go 1.19+)](#headings-go-119)
  - [Code blocks](#code-blocks)
  - [Lists](#lists)
  - [Links](#links)
- [Deprecation](#deprecation)
- [Known Bugs](#known-bugs)
- [godoc CLI and Tools](#godoc-cli-and-tools)

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

