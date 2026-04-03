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

## Docstring Styles

Python has four mainstream docstring styles. **Pick one and use it consistently project-wide.**

| Style | Best For | Tool Support |
|-------|----------|-------------|
| **Google** | General purpose, most readable inline | Sphinx + Napoleon |
| **NumPy** | Scientific computing, long sections | Sphinx + Napoleon |
| **reStructuredText (Sphinx)** | Sphinx-first projects, rich linking | Sphinx native |
| **Epytext** | Legacy; not recommended for new projects | Epydoc |

---

## Google Style (Recommended for most projects)

### Module docstring

```python
"""User authentication utilities.

This module provides JWT-based authentication helpers for FastAPI and Flask
applications. It handles token creation, validation, and refresh flows.

Typical usage::

    from mylib.auth import create_token, verify_token

    token = create_token(user_id=123, expires_in=3600)
    payload = verify_token(token)
    print(payload['sub'])  # '123'

Note:
    Tokens are signed using RS256. The signing key must be set via the
    ``AUTH_PRIVATE_KEY`` environment variable.

Todo:
    * Add support for revocation via Redis blocklist.
    * Implement PKCE flow for public clients.
"""
```

### Function docstring

```python
def create_token(
    user_id: int,
    expires_in: int = 3600,
    scopes: list[str] | None = None,
) -> str:
    """Creates a signed JWT access token for the given user.

    Generates a token containing the user ID as the subject claim,
    an expiration time, and optional OAuth2-style scope claims.

    Args:
        user_id: The numeric identifier of the authenticated user.
        expires_in: Token lifetime in seconds. Defaults to 3600 (1 hour).
            Must be positive. Use 0 for tokens that never expire (not
            recommended for production).
        scopes: List of OAuth2 scope strings to embed in the token.
            If None, no scope claim is included. Defaults to None.

    Returns:
        A compact, URL-safe JWT string (header.payload.signature format).

    Raises:
        ValueError: If ``user_id`` is not a positive integer.
        RuntimeError: If ``AUTH_PRIVATE_KEY`` is not set in the environment.
        jwt.exceptions.InvalidKeyError: If the private key is malformed.

    Example:
        >>> token = create_token(user_id=42, scopes=['read', 'write'])
        >>> payload = verify_token(token)
        >>> payload['sub']
        '42'
        >>> payload['scopes']
        ['read', 'write']
    """
```

### Class docstring

```python
class ConnectionPool:
    """Manages a pool of reusable database connections.

    Connections are created lazily on first use and returned to the pool
    after each context manager exit. Idle connections older than
    ``idle_timeout`` are evicted by a background thread.

    Attributes:
        max_size: Maximum number of simultaneous connections.
        idle_timeout: Seconds before an idle connection is closed.
        host: The database host address.
        port: The database port number.

    Example:
        >>> pool = ConnectionPool(host='localhost', max_size=5)
        >>> with pool.acquire() as conn:
        ...     result = conn.execute('SELECT 1')
        ...     result.fetchone()
        (1,)

    Note:
        This class is thread-safe. The same pool instance can be shared
        across multiple threads without external synchronization.
    """

    def __init__(
        self,
        host: str,
        port: int = 5432,
        max_size: int = 10,
        idle_timeout: float = 30.0,
    ) -> None:
        """Initializes the connection pool.

        Args:
            host: Hostname or IP address of the database server.
            port: TCP port the server is listening on. Defaults to 5432.
            max_size: Maximum number of connections to maintain.
                Additional requests block until a connection is returned.
                Defaults to 10.
            idle_timeout: Seconds before an unused connection is closed.
                Set to ``float('inf')`` to disable idle eviction.
                Defaults to 30.0.

        Raises:
            ValueError: If ``max_size`` < 1 or ``idle_timeout`` < 0.
            ConnectionError: If the initial ping to the server fails.
        """
```

### Method docstring

```python
def acquire(self, timeout: float = 5.0) -> Iterator[Connection]:
    """Acquires a connection from the pool as a context manager.

    Blocks until a connection is available or ``timeout`` is exceeded.
    The connection is automatically returned to the pool on context exit,
    even if an exception occurs inside the ``with`` block.

    Args:
        timeout: Maximum seconds to wait for a connection.
            Defaults to 5.0. Use ``float('inf')`` to wait indefinitely.

    Yields:
        A live database connection ready for use.

    Raises:
        TimeoutError: If no connection becomes available within ``timeout``.
        PoolExhaustedError: If the pool is closed and no connections remain.

    Example:
        >>> with pool.acquire(timeout=10.0) as conn:
        ...     rows = conn.execute('SELECT * FROM users').fetchall()
    """
```

### Property docstring

```python
@property
def size(self) -> int:
    """The current number of connections in the pool (active + idle).

    This count includes connections that are currently in use by
    callers and connections waiting in the idle pool.

    Returns:
        Non-negative integer. Always <= ``max_size``.
    """
    return self._active + len(self._idle)
```

### Generator and iterator docstring

```python
def stream_records(
    query: str,
    batch_size: int = 100,
) -> Generator[dict[str, Any], None, None]:
    """Streams database records one at a time using cursor-based pagination.

    Fetches records in batches of ``batch_size`` from the database and
    yields them individually. Memory usage is bounded by ``batch_size``
    regardless of total result count.

    Args:
        query: SQL SELECT statement to execute. Must not include LIMIT/OFFSET
            clauses — these are added automatically for pagination.
        batch_size: Number of records to fetch per database roundtrip.
            Larger values reduce latency; smaller values reduce memory.
            Defaults to 100.

    Yields:
        One record per iteration as a dictionary mapping column names to
        Python-native values (str, int, float, datetime, None, etc.).

    Raises:
        DatabaseError: If the query is malformed or the connection drops.

    Example:
        >>> for record in stream_records('SELECT * FROM events', batch_size=500):
        ...     process(record)
    """
```

---

## NumPy Style (For scientific/data projects)

```python
def pearson_correlation(x: np.ndarray, y: np.ndarray) -> float:
    """Computes the Pearson correlation coefficient between two arrays.

    Parameters
    ----------
    x : np.ndarray
        First data array. Must be 1-dimensional and non-empty.
        NaN values are not handled — use :func:`nanpearson` for NaN-aware
        computation.
    y : np.ndarray
        Second data array. Must have the same shape as ``x``.

    Returns
    -------
    float
        Pearson r coefficient in the range [-1.0, 1.0].
        Returns NaN if either array has zero variance.

    Raises
    ------
    ValueError
        If ``x`` and ``y`` have different shapes.
    ValueError
        If either array is empty or not 1-dimensional.

    Notes
    -----
    The Pearson coefficient measures linear correlation. For monotonic
    but non-linear relationships, use :func:`spearman_correlation` instead.

    Implementation uses the numerically stable two-pass algorithm.

    Examples
    --------
    Perfect positive correlation:

    >>> x = np.array([1, 2, 3, 4, 5])
    >>> y = np.array([2, 4, 6, 8, 10])
    >>> pearson_correlation(x, y)
    1.0

    No correlation:

    >>> x = np.array([1, 2, 3])
    >>> y = np.array([3, 1, 2])
    >>> round(pearson_correlation(x, y), 4)
    -0.5

    See Also
    --------
    spearman_correlation : Rank-based correlation coefficient.
    kendall_tau : Non-parametric correlation measure.
    """
```

---

## reStructuredText / Sphinx Style

```python
def send_email(
    to: str,
    subject: str,
    body: str,
    attachments: list[Path] | None = None,
) -> bool:
    """Send an email via the configured SMTP server.

    Sends a UTF-8 encoded email. Attachments are MIME-encoded inline.
    Returns False instead of raising if the SMTP connection fails,
    allowing callers to implement their own retry logic.

    :param to: Recipient email address. Must be a valid RFC 5321 address.
    :type to: str
    :param subject: Email subject line. HTML is not rendered.
    :type subject: str
    :param body: Plain-text email body. Use ``\\n`` for line breaks.
    :type body: str
    :param attachments: Optional list of file paths to attach.
        Files must exist and be readable. Defaults to None.
    :type attachments: list[Path] | None
    :returns: True if the email was accepted by the SMTP server,
        False if the server connection failed.
    :rtype: bool
    :raises ValueError: If ``to`` is not a valid email address.
    :raises FileNotFoundError: If any path in ``attachments`` does not exist.

    .. code-block:: python

        success = send_email(
            to='user@example.com',
            subject='Welcome!',
            body='Thanks for signing up.',
        )
        if not success:
            logger.warning('Email delivery failed — will retry later')

    .. note::
        SMTP credentials are read from ``SMTP_HOST``, ``SMTP_USER``,
        and ``SMTP_PASSWORD`` environment variables.

    .. deprecated:: 2.0.0
        Use :func:`send_email_async` for non-blocking delivery.
    """
```

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

## Doctest Integration

Doctests in docstrings are executed by `pytest --doctest-modules` or `python -m doctest`:

```python
def fibonacci(n: int) -> int:
    """Returns the nth Fibonacci number (0-indexed).

    Args:
        n: Non-negative integer index.

    Returns:
        The nth Fibonacci number.

    Raises:
        ValueError: If n is negative.

    Example:
        >>> fibonacci(0)
        0
        >>> fibonacci(1)
        1
        >>> fibonacci(10)
        55
        >>> fibonacci(-1)
        Traceback (most recent call last):
            ...
        ValueError: n must be non-negative
    """
```

---

## Type Hints and Documentation Interplay

With PEP 484 type hints, **omit type info from docstring param descriptions** — it's redundant:

```python
# ❌ Redundant — type already in signature
def process(
    items: list[str],
    max_count: int = 100,
) -> dict[str, int]:
    """Processes items.

    Args:
        items (list[str]): List of string items.    ← redundant
        max_count (int): Max count. Defaults to 100. ← redundant
    """

# ✅ Correct — describe semantics, not types
def process(
    items: list[str],
    max_count: int = 100,
) -> dict[str, int]:
    """Counts occurrences of each unique item.

    Args:
        items: Strings to count. Duplicates are tallied.
            Empty strings are included in the count.
        max_count: Maximum number of distinct items to track.
            Items beyond this limit are grouped under the key ``'__other__'``.
            Defaults to 100.

    Returns:
        Mapping of item strings to their occurrence counts.
        Always includes ``'__other__'`` key (possibly 0) when ``max_count`` is set.
    """
```

---

## Class Attribute Documentation

```python
class Config:
    """Application configuration loaded from environment variables.

    Attributes:
        debug: If True, enables verbose logging and disables caching.
        database_url: Full PostgreSQL connection string including credentials.
        secret_key: 32-byte hex string used for HMAC signing. Must be set.
        allowed_origins: List of CORS-allowed origin URLs.
            Defaults to empty (no CORS headers sent).
        rate_limit: Max requests per user per minute. 0 disables limiting.
    """

    debug: bool = False
    database_url: str
    secret_key: str
    allowed_origins: list[str] = field(default_factory=list)
    rate_limit: int = 60
```

---

## Module-Level `__all__` Documentation

```python
"""Image processing utilities.

This module provides functions for resizing, cropping, and converting images.
All functions accept and return PIL.Image objects unless otherwise noted.
"""

from __future__ import annotations

__all__ = [
    'resize',
    'crop',
    'convert_format',
    'ImageError',
]
# Only items in __all__ are included in `help(module)` and autodoc output.
```

---

## Sphinx Configuration

```python
# docs/conf.py
extensions = [
    'sphinx.ext.autodoc',       # generates docs from docstrings
    'sphinx.ext.napoleon',      # supports Google and NumPy styles
    'sphinx.ext.viewcode',      # adds source code links
    'sphinx.ext.intersphinx',   # cross-references to other projects
    'sphinx.ext.doctest',       # runs >>> doctests in docs
    'sphinx_autodoc_typehints', # renders PEP 484 type hints
]

# Napoleon settings for Google/NumPy style
napoleon_google_docstring = True
napoleon_numpy_docstring = True
napoleon_include_init_with_doc = True
napoleon_include_private_with_doc = False
napoleon_use_admonition_for_examples = True
napoleon_use_ivar = True

intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
    'numpy': ('https://numpy.org/doc/stable', None),
}
```

---

## Enforcing Documentation with Tools

```toml
# pyproject.toml — pydocstyle
[tool.pydocstyle]
convention = "google"  # or "numpy", "pep257"

# pyproject.toml — ruff (preferred over pydocstyle)
[tool.ruff.lint]
select = ["D"]  # pydocstyle rules
[tool.ruff.lint.pydocstyle]
convention = "google"
```

```bash
# Check docstrings
ruff check --select D .

# Run doctests
python -m doctest mymodule.py -v
pytest --doctest-modules src/

# Generate Sphinx docs
cd docs && make html
```

---

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
