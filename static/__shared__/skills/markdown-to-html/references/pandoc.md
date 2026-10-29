# pandoc

Reference for converting Markdown to HTML (and other formats) with [pandoc](https://github.com/jgm/pandoc).

## Contents

- [Quick Conversion Methods](#quick-conversion-methods)
  - [Method 1: CLI Basic Conversion](#method-1-cli-basic-conversion)
  - [Method 2: Filter Mode (Interactive)](#method-2-filter-mode-interactive)
  - [Method 3: Format Conversion](#method-3-format-conversion)
- [CLI Configuration](#cli-configuration)
- [Security Warning](#security-warning)
- [Supported Markdown Flavors](#supported-markdown-flavors)
- [Troubleshooting](#troubleshooting)

### Quick Conversion Methods

#### Method 1: CLI Basic Conversion

```bash
# Convert markdown to HTML
pandoc input.md -o output.html

# Convert with standalone document (includes header/footer)
pandoc input.md -s -o output.html

# Explicit format specification
pandoc input.md -f markdown -t html -s -o output.html
```

#### Method 2: Filter Mode (Interactive)

```bash
# Start pandoc as a filter
pandoc

# Type markdown, then Ctrl-D (Linux/macOS) or Ctrl-Z+Enter (Windows)
Hello *pandoc*!
# Output: <p>Hello <em>pandoc</em>!</p>
```

#### Method 3: Format Conversion

```bash
# HTML to Markdown
pandoc -f html -t markdown input.html -o output.md

# Markdown to LaTeX
pandoc input.md -s -o output.tex

# Markdown to PDF (requires LaTeX)
pandoc input.md -s -o output.pdf

# Markdown to Word
pandoc input.md -s -o output.docx
```

### CLI Configuration

| Option                     | Description                                    |
| -------------------------- | ---------------------------------------------- |
| `-f, --from <format>`      | Input format (markdown, html, latex, etc.)     |
| `-t, --to <format>`        | Output format (html, latex, pdf, docx, etc.)   |
| `-s, --standalone`         | Produce standalone document with header/footer |
| `-o, --output <file>`      | Output file (inferred from extension)          |
| `--mathml`                 | Convert TeX math to MathML                     |
| `--metadata title="Title"` | Set document metadata                          |
| `--toc`                    | Include table of contents                      |
| `--template <file>`        | Use custom template                            |
| `--help`                   | Show all options                               |

### Security Warning

⚠️ **Pandoc processes input faithfully.** When converting untrusted markdown:

- Use `--sandbox` mode to disable external file access
- Validate input before processing
- Sanitize HTML output if displayed in browsers

```bash
# Run in sandbox mode for untrusted input
pandoc --sandbox input.md -o output.html
```

### Supported Markdown Flavors

| Flavor                   | Support                    |
| ------------------------ | -------------------------- |
| Pandoc Markdown          | 100% (native)              |
| CommonMark               | Full (use `-f commonmark`) |
| GitHub Flavored Markdown | Full (use `-f gfm`)        |
| MultiMarkdown            | Partial                    |

### Troubleshooting

| Issue                      | Solution                                         |
| -------------------------- | ------------------------------------------------ |
| PDF generation fails       | Install LaTeX (MacTeX, MiKTeX, or texlive)       |
| Encoding issues on Windows | Run `chcp 65001` before using pandoc             |
| Missing standalone headers | Add `-s` flag for complete documents             |
| Math not rendering         | Use `--mathml` or `--mathjax` option             |
| Tables not rendering       | Ensure proper table syntax with pipes and dashes |

