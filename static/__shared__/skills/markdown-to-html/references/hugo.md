# Hugo

Reference for building and converting Markdown content with [gohugoio/hugo](https://github.com/gohugoio/hugo).

## Contents

- [Quick Conversion Methods](#quick-conversion-methods)
  - [Method 1: Create New Site](#method-1-create-new-site)
  - [Method 2: Build Static Site](#method-2-build-static-site)
  - [Method 3: Development Server](#method-3-development-server)
- [CLI Configuration](#cli-configuration)
- [Security Warning](#security-warning)
- [Supported Markdown Flavors](#supported-markdown-flavors)
- [Troubleshooting](#troubleshooting)

### Quick Conversion Methods

#### Method 1: Create New Site

```bash
# Create a new Hugo site
hugo new site mysite

# Change to site directory
cd mysite

# Add a theme
git init
git submodule add https://github.com/theNewDynamic/gohugo-theme-ananke themes/ananke
echo "theme = 'ananke'" >> hugo.toml

# Create content
hugo new content posts/my-first-post.md

# Start development server
hugo server -D
```

#### Method 2: Build Static Site

```bash
# Build site to public directory
hugo

# Build with minification
hugo --minify

# Build for specific environment
hugo --environment production
```

#### Method 3: Development Server

```bash
# Start server with drafts
hugo server -D

# Start with live reload and bind to all interfaces
hugo server --bind 0.0.0.0 --baseURL http://localhost:1313/

# Start with specific port
hugo server --port 8080
```

### CLI Configuration

| Command                   | Description                      |
| ------------------------- | -------------------------------- |
| `hugo new site <name>`    | Create new Hugo site             |
| `hugo new content <path>` | Create new content file          |
| `hugo`                    | Build site to `public` directory |
| `hugo server`             | Start development server         |
| `hugo mod init`           | Initialize Hugo Modules          |

| Build Options              | Description                        |
| -------------------------- | ---------------------------------- |
| `-D, --buildDrafts`        | Include draft content              |
| `-E, --buildExpired`       | Include expired content            |
| `-F, --buildFuture`        | Include future-dated content       |
| `--minify`                 | Minify output                      |
| `--gc`                     | Run garbage collection after build |
| `-d, --destination <path>` | Output directory                   |

| Server Options            | Description                 |
| ------------------------- | --------------------------- |
| `--bind <ip>`             | Interface to bind to        |
| `-p, --port <port>`       | Port number (default: 1313) |
| `--liveReloadPort <port>` | Live reload port            |
| `--disableLiveReload`     | Disable live reload         |
| `--navigateToChanged`     | Navigate to changed content |

### Security Warning

⚠️ **Hugo security considerations:**

- Configure security policy in `hugo.toml` for external commands
- Use `--enableGitInfo` carefully with public repositories
- Validate shortcode parameters for user-generated content

```toml
# hugo.toml security settings
[security]
  enableInlineShortcodes = false
  [security.exec]
    allow = ['^go$', '^npx$', '^postcss$']
  [security.funcs]
    getenv = ['^HUGO_', '^CI$']
  [security.http]
    methods = ['(?i)GET|POST']
    urls = ['.*']
```

### Supported Markdown Flavors

| Flavor                   | Support                                 |
| ------------------------ | --------------------------------------- |
| Goldmark (default)       | 100% (CommonMark compliant)             |
| GitHub Flavored Markdown | Full (tables, strikethrough, autolinks) |
| CommonMark               | 100%                                    |
| Blackfriday (legacy)     | Deprecated, not recommended             |

Configure markdown in `hugo.toml`:

```toml
[markup]
  [markup.goldmark]
    [markup.goldmark.extensions]
      definitionList = true
      footnote = true
      linkify = true
      strikethrough = true
      table = true
      taskList = true
    [markup.goldmark.renderer]
      unsafe = false  # Set true to allow raw HTML
```

### Troubleshooting

| Issue                     | Solution                                        |
| ------------------------- | ----------------------------------------------- |
| "Page not found" on paths | Check `baseURL` in config                       |
| Theme not loading         | Verify theme in `themes/` or Hugo Modules       |
| Slow builds               | Use `--templateMetrics` to identify bottlenecks |
| Raw HTML not rendering    | Set `unsafe = true` in goldmark config          |
| Images not loading        | Check `static/` folder structure                |
| Module errors             | Run `hugo mod tidy`                             |

