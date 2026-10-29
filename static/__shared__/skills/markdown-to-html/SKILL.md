---
name: markdown-to-html
description: 'Convert Markdown files to HTML similar to `marked.js`, `pandoc`, `gomarkdown/markdown`, or similar tools; or writing custom script to convert markdown to html and/or working on web template systems like `jekyll/jekyll`, `gohugoio/hugo`, or similar web templating systems that utilize markdown documents, converting them to html. Use when asked to "convert markdown to html", "transform md to html", "render markdown", "generate html from markdown", or when working with .md files and/or web a templating system that converts markdown to HTML output. Supports CLI and Node.js workflows with GFM, CommonMark, and standard Markdown flavors.'
---

# Markdown to HTML Conversion

Expert skill for converting Markdown documents to HTML using the marked.js library, or writing data conversion scripts; in this case scripts similar to [markedJS/marked](https://github.com/markedjs/marked) repository. For custom scripts knowledge is not confined to `marked.js`, but data conversion methods are utilized from tools like [pandoc](https://github.com/jgm/pandoc) and [gomarkdown/markdown](https://github.com/gomarkdown/markdown) for data conversion; [jekyll/jekyll](https://github.com/jekyll/jekyll) and [gohugoio/hugo](https://github.com/gohugoio/hugo) for templating systems.

The conversion script or tool should handle single files, batch conversions, and advanced configurations.

## When to Use This Skill

- User asks to "convert markdown to html" or "transform md files"
- User wants to "render markdown" as HTML output
- User needs to generate HTML documentation from .md files
- User is building static sites from Markdown content
- User is building template system that converts markdown to html
- User is working on a tool, widget, or custom template for an existing templating system
- User wants to preview Markdown as rendered HTML


## Converting Markdown to HTML

### Essential Basic Conversions

For more see [basic-markdown.md](references/basic-markdown.md)

### Code Block Conversions

For more see [code-blocks.md](references/code-blocks.md)

### Collapsed Section Conversions

For more see [collapsed-sections.md](references/collapsed-sections.md)

### Mathematical Expression Conversions

For more see [writing-mathematical-expressions.md](references/writing-mathematical-expressions.md)

### Table Conversions

For more see [tables.md](references/tables.md)


## Working with [`markedJS/marked`](references/marked.md)

### Prerequisites

- Node.js installed (for CLI or programmatic usage)
- Install marked globally for CLI: `npm install -g marked`
- Or install locally: `npm install marked`

### Quick Conversion Methods

See [marked.md](references/marked.md) **Quick Conversion Methods**

### Step-by-Step Workflows

See [marked.md](references/marked.md) **Step-by-Step Workflows**


### CLI Configuration, Security, Flavors, and Troubleshooting

See [marked.md](references/marked.md).

## Working with [`pandoc`](references/pandoc.md)

### Prerequisites

- Pandoc installed (download from <https://pandoc.org/installing.html>)
- For PDF output: LaTeX installation (MacTeX on macOS, MiKTeX on Windows, texlive on Linux)
- Terminal/command prompt access


### Quick Conversion Methods, CLI Configuration, Security, Flavors, and Troubleshooting

See [pandoc.md](references/pandoc.md).

## Working with [`gomarkdown/markdown`](references/gomarkdown.md)

### Prerequisites

- Go 1.18 or higher installed
- Install the library: `go get github.com/gomarkdown/markdown`
- For CLI tool: `go install github.com/gomarkdown/mdtohtml@latest`


### Quick Conversion Methods, CLI Configuration, Security, Flavors, and Troubleshooting

See [gomarkdown.md](references/gomarkdown.md).

## Working with [`jekyll`](references/jekyll.md)

### Prerequisites

- Ruby version 2.7.0 or higher
- RubyGems
- GCC and Make (for native extensions)
- Install Jekyll and Bundler: `gem install jekyll bundler`


### Quick Conversion Methods, CLI Configuration, Security, Flavors, and Troubleshooting

See [jekyll.md](references/jekyll.md).

## Working with [`hugo`](references/hugo.md)

### Prerequisites

- Hugo installed (download from <https://gohugo.io/installation/>)
- Git (recommended for themes and modules)
- Go (optional, for Hugo Modules)


### Quick Conversion Methods, CLI Configuration, Security, Flavors, and Troubleshooting

See [hugo.md](references/hugo.md).

## References

### Writing and Styling Markdown

- [basic-markdown.md](references/basic-markdown.md)
- [code-blocks.md](references/code-blocks.md)
- [collapsed-sections.md](references/collapsed-sections.md)
- [tables.md](references/tables.md)
- [writing-mathematical-expressions.md](references/writing-mathematical-expressions.md)
- Markdown Guide: <https://www.markdownguide.org/basic-syntax/>
- Styling Markdown: <https://github.com/sindresorhus/github-markdown-css>

### [`markedJS/marked`](references/marked.md)

- Official documentation: <https://marked.js.org/>
- Advanced options: <https://marked.js.org/using_advanced>
- Extensibility: <https://marked.js.org/using_pro>
- GitHub repository: <https://github.com/markedjs/marked>

### [`pandoc`](references/pandoc.md)

- Getting started: <https://pandoc.org/getting-started.html>
- Official documentation: <https://pandoc.org/MANUAL.html>
- Extensibility: <https://pandoc.org/extras.html>
- GitHub repository: <https://github.com/jgm/pandoc>

### [`gomarkdown/markdown`](references/gomarkdown.md)

- Official documentation: <https://pkg.go.dev/github.com/gomarkdown/markdown>
- Advanced configuration: <https://pkg.go.dev/github.com/gomarkdown/markdown@v0.0.0-20250810172220-2e2c11897d1a/html>
- Markdown processing: <https://blog.kowalczyk.info/article/cxn3/advanced-markdown-processing-in-go.html>
- GitHub repository: <https://github.com/gomarkdown/markdown>

### [`jekyll`](references/jekyll.md)

- Official documentation: <https://jekyllrb.com/docs/>
- Configuration options: <https://jekyllrb.com/docs/configuration/options/>
- Plugins: <https://jekyllrb.com/docs/plugins/>
  - [Installation](https://jekyllrb.com/docs/plugins/installation/)
  - [Generators](https://jekyllrb.com/docs/plugins/generators/)
  - [Converters](https://jekyllrb.com/docs/plugins/converters/)
  - [Commands](https://jekyllrb.com/docs/plugins/commands/)
  - [Tags](https://jekyllrb.com/docs/plugins/tags/)
  - [Filters](https://jekyllrb.com/docs/plugins/filters/)
  - [Hooks](https://jekyllrb.com/docs/plugins/hooks/)
- GitHub repository: <https://github.com/jekyll/jekyll>

### [`hugo`](references/hugo.md)

- Official documentation: <https://gohugo.io/documentation/>
- All Settings: <https://gohugo.io/configuration/all/>
- Editor Plugins: <https://gohugo.io/tools/editors/>
- GitHub repository: <https://github.com/gohugoio/hugo>
