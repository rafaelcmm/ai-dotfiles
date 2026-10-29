# Advanced Documentation Topics

Doctest integration, type-hint/docstring interplay, class attribute and
`__all__` documentation, Sphinx configuration, and tool-based enforcement.

## Contents

- [Doctest Integration](#doctest-integration)
- [Type Hints and Documentation Interplay](#type-hints-and-documentation-interplay)
- [Class Attribute Documentation](#class-attribute-documentation)
- [Module-Level `__all__` Documentation](#module-level-all-documentation)
- [Sphinx Configuration](#sphinx-configuration)
- [Enforcing Documentation with Tools](#enforcing-documentation-with-tools)

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

