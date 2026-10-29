---
name: python-documentation-best-practices
description: >
  Comprehensive guide for documenting Python code with docstrings, inline comments,
  type hints, and Sphinx/autodoc. Use this skill whenever writing, reviewing, or improving
  Python documentation — including module docstrings, class docstrings, function and method
  docstrings, property documentation, type annotations, Google/NumPy/reST docstring styles,
  sphinx configuration, doctest, and inline comments. Trigger on any .py file that needs
  documentation added, reviewed, or improved — including libraries, scripts, Django/Flask apps,
  FastAPI services, data science notebooks, CLI tools, and any Python project.
---

# Python Documentation Best Practices

## Philosophy

Python has two complementary documentation tools: **docstrings** (introspectable at runtime)
and **comments** (source-only). Use docstrings for API documentation — they appear in `help()`,
IDEs, and doc generators. Use `#` comments to explain implementation decisions.

**Core rules (PEP 257):**
1. All public modules, classes, functions, and methods must have docstrings.
2. Use `"""triple double quotes"""` — always, even for one-liners.
3. One-liners: closing `"""` on the same line as the opening.
4. Multi-line: summary line, blank line, body, closing `"""` on its own line.
5. Docstrings describe behavior — not implementation. The `#` comment explains the how.

---

## Scope and restraint

Canonical policy: `commenting-standards`. This section applies it to docstrings.

### Coverage is absolute, length is proportional

Every public module, class, function and method keeps its docstring — PEP 257
and the Core rules above are not relaxed. What scales is depth, never presence.
Depth is owed in proportion to visibility, subtlety, and how far a mistake
travels.

**A one-line docstring is a complete docstring** when the signature and type
hints already carry the contract. PEP 257 endorses this explicitly, and the type
hints do real work here: annotated parameters need no `Args:` entry restating
their type.

```python
def is_expired(token: Token) -> bool:
    """Return True if the token's expiry has passed."""
```

That is finished documentation, not a stub. Expanding it to a four-section
Google-style block with `Args:`, `Returns:` and `Examples:` would add lines and
no facts. Padding is a defect, not thoroughness.

### Structured sections stay

The chosen style's sections are the contract: `Args:` / `Returns:` / `Raises:`
in Google style, `Parameters` / `Returns` / `Raises` in NumPy, and `:param:` /
`:returns:` / `:raises:` in Sphinx. Sphinx, Napoleon, `help()` and every IDE
consume them.

Brevity applies to a section's **content**, never to its **presence**. Once a
docstring needs sections at all, omitting `Raises:` from a function that raises
removes a fact a caller needs. Make each entry carry what the annotation cannot —
units, valid ranges, edge values, what an empty return means.

```diff
-Args:
-    timeout: The timeout.
-Returns:
-    A dict.
+Args:
+    timeout: Seconds to wait per attempt. Must be positive; 0 raises.
+Returns:
+    Decoded JSON body. An empty body yields ``{}``, not ``None``.
```

### Document the present

A docstring states what the object does now; it is not a change log. `git log`,
`git blame` and the pull request own history and keep it accurate for free. Never
write provenance (`Extracted from utils.py when...`), removed behaviour (`This
used to use a list`), dated claims, supersession notices, or ticket archaeology.
Such a comment goes wrong at the next edit, and it is the one kind of staleness a
reader cannot detect, because nothing in the file contradicts it.

The `TODO`/`FIXME` conventions documented later in this skill are the sanctioned
exceptions, and they work precisely because each is a present, actionable fact
rather than a record of the past: a `TODO` with a ticket reference, a `HACK` or
`FIXME` naming the upstream cause and the condition that retires it, a
`DEPRECATED` marker naming the replacement, and an invariant with its expiry
trigger. Keep those in `#` comments; keep narrative history out of both.

### Assertive voice and the stop rule

PEP 257 asks for the imperative mood — `"""Return the slug."""`, not
`"""Returns the slug."""` or `"""This function will return the slug."""`. Cut
hedges (`basically`, `it is important to note`) and self-reference. A docstring
is finished when a competent caller can use the object correctly without reading
its body; implementation reasoning goes in a `#` comment at the line it
explains, and history goes nowhere.

### Before and after

```python
def connect(dsn: str, timeout: float = 5.0) -> Connection:
    """This function is basically a helper which is used in order to open a
    database connection.

    It is important to note that it returns a Connection. Originally this was
    in db/legacy.py; it was moved here during the 2026 Q3 refactor, and
    previously the timeout defaulted to 30 seconds.

    Args:
        dsn: The dsn, which is a string.
        timeout: The timeout, which is a float.

    Returns:
        A Connection object.
    """
```

```python
def connect(dsn: str, timeout: float = 5.0) -> Connection:
    """Open a pooled database connection.

    Args:
        dsn: Standard connection URI. Credentials in it are never logged.
        timeout: Seconds to wait per attempt, not for the whole call. Retries
            are not performed here; the caller owns the retry policy.

    Returns:
        A connection checked out from the shared pool. The caller must close
        it, or use it as a context manager, or the pool leaks a slot.

    Raises:
        TimeoutError: If no connection becomes available within ``timeout``.
        ValueError: If ``dsn`` is malformed.
    """
```

Shorter because the padding left, not because information did. Every section
survived, `Raises:` appeared because the function raises, and the caller now
knows about the pool-slot obligation.

---

## Docstring Styles

Python has four mainstream docstring styles. **Pick one and use it consistently project-wide.**

| Style | Best For | Tool Support |
|-------|----------|-------------|
| **Google** | General purpose, most readable inline | Sphinx + Napoleon |
| **NumPy** | Scientific computing, long sections | Sphinx + Napoleon |
| **reStructuredText (Sphinx)** | Sphinx-first projects, rich linking | Sphinx native |
| **Epytext** | Legacy; not recommended for new projects | Epydoc |

---


Full worked examples for each style (module, function, class, method, property,
generator docstrings) are in [references/docstring-styles.md](references/docstring-styles.md).

---

## Inline Comments Best Practices

### Explain WHY, not WHAT

```python
# ✅ Good — explains intent that the code alone doesn't convey
# Use a 25% growth factor instead of doubling to avoid wasting too much memory
# on large arrays. Python's list uses a similar heuristic (over-allocate by ~12%).
new_capacity = current + max(current // 4, 1)

# ✅ Good — explains non-obvious business rule
# Subtract 1 from the day to convert from 1-indexed (display) to 0-indexed (internal).
# This is an intentional off-by-one — do not "fix" it.
day_index = user_input_day - 1

# ✅ Good — explains a workaround
# Workaround for https://bugs.python.org/issue43517: datetime.fromisoformat does
# not handle 'Z' as UTC suffix in Python < 3.11. Replace manually.
timestamp_str = raw.replace('Z', '+00:00')

# ❌ Bad — restates the code
x = x + 1  # increment x by 1
users = []  # create empty list of users
```

### TODO/FIXME with context

```python
# TODO(alice): Replace this O(n²) algorithm with a hash join once the
# table sizes exceed 10k rows in production (see TICKET-4521).

# FIXME: Race condition if two requests hit this simultaneously.
# Need a distributed lock — see the Redis redlock pattern.

# HACK: The vendor API returns 200 OK for validation errors. Detect
# errors by checking if the body contains the 'error' key.

# NOTE: Must be called before logging is configured — do not move this line.
# DEPRECATED: Remove after migration to v2 is complete (target: 2024-Q1).
```

---


## Advanced Topics

Doctest integration, type-hint/docstring interplay, class attribute and
`__all__` documentation, Sphinx configuration, and enforcement tooling
(ruff/pydocstyle) are in [references/advanced-topics.md](references/advanced-topics.md).

## Anti-Patterns

```python
# ❌ Single quotes — use triple double quotes
def greet(name):
    'Returns a greeting.'

# ❌ No docstring on public function
def calculate_tax(income, rate):
    return income * rate

# ❌ Docstring as comment (doesn't follow the function)
def process():
    result = compute()
    """This processes the data."""  # ← not actually a docstring; just a string literal
    return result

# ❌ Vague summary that doesn't say what the function does
def handle_request(request):
    """Handles the request."""  # ← tells us nothing

# ✅ Correct
def handle_request(request: HttpRequest) -> HttpResponse:
    """Processes an HTTP request and returns an appropriate HTTP response.

    Dispatches to the appropriate controller based on the request method
    and path. Returns 405 Method Not Allowed for unsupported methods.
    ...
    """

# ❌ Docstring placed after code (not the first statement)
def connect(host):
    sock = socket.create_connection(host)
    """Connects to host."""  # ← not a docstring — Python doesn't see it
    return sock

# ❌ Missing Raises section when exceptions can propagate
def load_config(path: str) -> dict:
    """Loads configuration from a JSON file."""
    with open(path) as f:  # raises FileNotFoundError — not documented!
        return json.load(f)  # raises json.JSONDecodeError — not documented!
```

---

## Documentation Checklist

- [ ] All public modules have a module-level docstring with summary and usage example
- [ ] All public classes have a class docstring with `Attributes:` section
- [ ] `__init__` documents all constructor parameters under `Args:`
- [ ] All public functions/methods have docstrings with `Args:`, `Returns:`, `Raises:`
- [ ] `Args:` documents every parameter — name, description, constraints, defaults
- [ ] `Returns:` describes the meaning of the return value (not just its type)
- [ ] `Raises:` lists every exception that can propagate out of the function
- [ ] `Example:` or `Examples:` present for all non-trivial public functions
- [ ] Doctest examples are accurate and runnable (`python -m doctest` passes)
- [ ] Type hints present on all public function signatures (PEP 484)
- [ ] Type info NOT duplicated in docstrings when PEP 484 hints are present
- [ ] One consistent docstring style used throughout the project (Google/NumPy/reST)
- [ ] `"""triple double quotes"""` used exclusively — no single or double quotes
- [ ] Multi-line docstrings: summary on first line, blank line, body, closing `"""` alone
- [ ] Inline `#` comments explain WHY — not WHAT
- [ ] TODO/FIXME comments include owner, context, and tracking issue
- [ ] No commented-out code without explanation
- [ ] `__all__` defined in modules to control public API surface
- [ ] pydocstyle or ruff `D` rules enabled in CI to enforce standards
