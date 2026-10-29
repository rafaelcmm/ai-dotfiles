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

## Scope and restraint

Canonical policy: `commenting-standards`. This section applies it to godoc.

### Coverage is absolute, length is proportional

Every exported identifier keeps its doc comment — the Core rules above are not
relaxed, and the comment still begins with the identifier's name and reads as a
complete sentence. What scales is depth, never presence. Depth is owed in
proportion to visibility, subtlety, and how far a mistake travels.

The rule stated below for packages — *for simple packages, one line is fine* — is
the right instinct and **generalizes to every exported identifier**. One
assertive sentence is a complete doc comment whenever the name and signature
already carry the contract:

```go
// Len returns the number of queued jobs.
func (q *Queue) Len() int
```

That is finished documentation, not a stub. Go's doc culture is deliberately
plain; a paragraph where a sentence suffices is padding, and padding is a defect.
Depth is for the identifier whose misuse is costly — concurrency obligations,
ownership of a returned value, what a nil or zero result means, which errors a
caller must distinguish.

### Document the present

A doc comment states what the identifier does now; it is not a change log. `git
log`, `git blame` and the pull request own history and keep it accurate for free.
Never write provenance (`Extracted from client.go when...`), removed behaviour
(`This used to take a string`), dated claims, supersession notices, or ticket
archaeology. Such a comment goes wrong at the next edit, and it is the one kind
of staleness a reader cannot detect, because nothing in the package contradicts
it.

Legitimately time-bearing, because each is present and actionable: a
`TODO(name)` with a tracking reference; a workaround naming its upstream issue
and the condition that retires it; the `// Deprecated:` paragraph godoc
recognizes, naming the replacement; and an invariant with its expiry trigger.

### Assertive voice and the stop rule

Lead with the identifier name and one declarative sentence in the present tense
indicative — `Parse reads...`, not `Parse will basically read...`. Cut hedges and
self-reference. A doc comment is finished when a competent caller can use the
identifier correctly without reading its body; implementation reasoning goes in a
`//` comment inside the body at the line it explains, and history goes nowhere.

### Before and after

```go
// ParsePort is basically a helper function which is used in order to parse a
// port number from a string. It is important to note that it returns an error
// as its second value. This function was originally in config/legacy.go but
// was moved here during the Q3 refactor, and previously it accepted port 0.
func ParsePort(s string) (uint16, error)
```

```go
// ParsePort parses a TCP port from s, rejecting 0.
//
// Port 0 asks the OS for an arbitrary free port. That is valid when binding a
// listener but never valid in configuration, where it silently yields a service
// on an address nobody can predict.
//
// The returned error wraps ErrReservedPort for 0 and ErrPortRange above 65535,
// so callers can distinguish a typo from a deliberate 0.
func ParsePort(s string) (uint16, error)
```

Shorter because the padding left, not because information did. The caller gained
the two facts the signature could not express: why 0 is refused, and which errors
are distinguishable.

---


## Documenting Constructs

Worked examples for comment types and for documenting packages, functions,
methods, types, and Example functions are in
[references/documenting-constructs.md](references/documenting-constructs.md).

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


## Godoc Formatting and Tooling

Godoc comment formatting rules, deprecation/known-bugs conventions, and
`go doc`/`godoc` commands are in
[references/godoc-and-tooling.md](references/godoc-and-tooling.md).

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
