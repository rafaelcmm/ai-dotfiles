# gomarkdown/markdown

Reference for converting Markdown to HTML with [gomarkdown/markdown](https://github.com/gomarkdown/markdown).

## Contents

- [Quick Conversion Methods](#quick-conversion-methods)
  - [Method 1: Simple Conversion (Go)](#method-1-simple-conversion-go)
  - [Method 2: CLI Tool](#method-2-cli-tool)
  - [Method 3: Custom Parser and Renderer](#method-3-custom-parser-and-renderer)
- [CLI Configuration](#cli-configuration)
- [Security Warning](#security-warning)
- [Supported Markdown Flavors](#supported-markdown-flavors)
- [Troubleshooting](#troubleshooting)

### Quick Conversion Methods

#### Method 1: Simple Conversion (Go)

```go
package main

import (
    "fmt"
    "github.com/gomarkdown/markdown"
)

func main() {
    md := []byte("# Hello World\n\nThis is **bold** text.")
    html := markdown.ToHTML(md, nil, nil)
    fmt.Println(string(html))
}
```

#### Method 2: CLI Tool

```bash
# Install mdtohtml
go install github.com/gomarkdown/mdtohtml@latest

# Convert file
mdtohtml input.md output.html

# Convert file (output to stdout)
mdtohtml input.md
```

#### Method 3: Custom Parser and Renderer

```go
package main

import (
    "github.com/gomarkdown/markdown"
    "github.com/gomarkdown/markdown/html"
    "github.com/gomarkdown/markdown/parser"
)

func mdToHTML(md []byte) []byte {
    // Create parser with extensions
    extensions := parser.CommonExtensions | parser.AutoHeadingIDs | parser.NoEmptyLineBeforeBlock
    p := parser.NewWithExtensions(extensions)
    doc := p.Parse(md)

    // Create HTML renderer with extensions
    htmlFlags := html.CommonFlags | html.HrefTargetBlank
    opts := html.RendererOptions{Flags: htmlFlags}
    renderer := html.NewRenderer(opts)

    return markdown.Render(doc, renderer)
}
```

### CLI Configuration

The `mdtohtml` CLI tool has minimal options:

```bash
mdtohtml input-file [output-file]
```

For advanced configuration, use the Go library programmatically with parser and renderer options:

| Parser Extension                | Description                                         |
| ------------------------------- | --------------------------------------------------- |
| `parser.CommonExtensions`       | Tables, fenced code, autolinks, strikethrough, etc. |
| `parser.AutoHeadingIDs`         | Generate IDs for headings                           |
| `parser.NoEmptyLineBeforeBlock` | No blank line needed before blocks                  |
| `parser.MathJax`                | MathJax support for LaTeX math                      |

| HTML Flag              | Description                    |
| ---------------------- | ------------------------------ |
| `html.CommonFlags`     | Common HTML output flags       |
| `html.HrefTargetBlank` | Add `target="_blank"` to links |
| `html.CompletePage`    | Generate complete HTML page    |
| `html.UseXHTML`        | Generate XHTML output          |

### Security Warning

⚠️ **gomarkdown does NOT sanitize output HTML.** For untrusted input, use Bluemonday:

```go
import (
    "github.com/microcosm-cc/bluemonday"
    "github.com/gomarkdown/markdown"
)

maybeUnsafeHTML := markdown.ToHTML(md, nil, nil)
html := bluemonday.UGCPolicy().SanitizeBytes(maybeUnsafeHTML)
```

Recommended sanitizer: [Bluemonday](https://github.com/microcosm-cc/bluemonday)

### Supported Markdown Flavors

| Flavor                   | Support                                   |
| ------------------------ | ----------------------------------------- |
| Original Markdown        | 100%                                      |
| CommonMark               | High (with extensions)                    |
| GitHub Flavored Markdown | High (tables, fenced code, strikethrough) |
| MathJax/LaTeX Math       | Supported via extension                   |
| Mmark                    | Supported                                 |

### Troubleshooting

| Issue                            | Solution                                      |
| -------------------------------- | --------------------------------------------- |
| Windows/Mac newlines not parsed  | Use `parser.NormalizeNewlines(input)`         |
| Tables not rendering             | Enable `parser.Tables` extension              |
| Code blocks without highlighting | Integrate with syntax highlighter like Chroma |
| Math not rendering               | Enable `parser.MathJax` extension             |
| XSS vulnerabilities              | Use Bluemonday to sanitize output             |

