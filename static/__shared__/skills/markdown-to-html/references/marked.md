# marked.js

Reference for converting Markdown to HTML with [markedJS/marked](https://github.com/markedjs/marked).

## Contents

- [CLI Configuration](#cli-configuration)
- [Using Config Files](#using-config-files)
- [CLI Options Reference](#cli-options-reference)
- [Security Warning](#security-warning)
- [Supported Markdown Flavors](#supported-markdown-flavors)
- [Troubleshooting](#troubleshooting)

### CLI Configuration

### Using Config Files

Create `~/.marked.json` for persistent options:

```json
{
  "gfm": true,
  "breaks": true
}
```

Or use a custom config:

```bash
marked -i input.md -o output.html -c config.json
```

### CLI Options Reference

| Option                  | Description                     |
| ----------------------- | ------------------------------- |
| `-i, --input <file>`    | Input Markdown file             |
| `-o, --output <file>`   | Output HTML file                |
| `-s, --string <string>` | Parse string instead of file    |
| `-c, --config <file>`   | Use custom config file          |
| `--gfm`                 | Enable GitHub Flavored Markdown |
| `--breaks`              | Convert newlines to `<br>`      |
| `--help`                | Show all options                |

### Security Warning

⚠️ **Marked does NOT sanitize output HTML.** For untrusted input, use a sanitizer:

```javascript
import { marked } from "marked";
import DOMPurify from "dompurify";

const unsafeHtml = marked.parse(untrustedMarkdown);
const safeHtml = DOMPurify.sanitize(unsafeHtml);
```

Recommended sanitizers:

- [DOMPurify](https://github.com/cure53/DOMPurify) (recommended)
- [sanitize-html](https://github.com/apostrophecms/sanitize-html)
- [js-xss](https://github.com/leizongmin/js-xss)

### Supported Markdown Flavors

| Flavor                   | Support |
| ------------------------ | ------- |
| Original Markdown        | 100%    |
| CommonMark 0.31          | 98%     |
| GitHub Flavored Markdown | 97%     |

### Troubleshooting

| Issue                            | Solution                                                                    |
| -------------------------------- | --------------------------------------------------------------------------- |
| Special characters at file start | Strip zero-width chars: `content.replace(/^[\u200B\u200C\u200D\uFEFF]/,"")` |
| Code blocks not highlighting     | Add a syntax highlighter like highlight.js                                  |
| Tables not rendering             | Ensure `gfm: true` option is set                                            |
| Line breaks ignored              | Set `breaks: true` in options                                               |
| XSS vulnerability concerns       | Use DOMPurify to sanitize output                                            |

