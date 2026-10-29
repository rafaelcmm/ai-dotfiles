# Docstring Style Walkthroughs

Full worked examples for each Python docstring style: module, function, class,
method, property, and generator docstrings. Read the one style section your
project uses.

## Contents

- [Google Style (Recommended for most projects)](#google-style-recommended-for-most-projects)
  - [Module docstring](#module-docstring)
  - [Function docstring](#function-docstring)
  - [Class docstring](#class-docstring)
  - [Method docstring](#method-docstring)
  - [Property docstring](#property-docstring)
  - [Generator and iterator docstring](#generator-and-iterator-docstring)
- [NumPy Style (For scientific/data projects)](#numpy-style-for-scientificdata-projects)
- [reStructuredText / Sphinx Style](#restructuredtext-sphinx-style)

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

