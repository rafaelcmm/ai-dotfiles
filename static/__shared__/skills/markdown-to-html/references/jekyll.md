# Jekyll

Reference for building and converting Markdown content with [jekyll/jekyll](https://github.com/jekyll/jekyll).

## Contents

- [Quick Conversion Methods](#quick-conversion-methods)
  - [Method 1: Create New Site](#method-1-create-new-site)
  - [Method 2: Build Static Site](#method-2-build-static-site)
  - [Method 3: Live Reload Development](#method-3-live-reload-development)
- [CLI Configuration](#cli-configuration)
- [Security Warning](#security-warning)
- [Supported Markdown Flavors](#supported-markdown-flavors)
- [Troubleshooting](#troubleshooting)

### Quick Conversion Methods

#### Method 1: Create New Site

```bash
# Create a new Jekyll site
jekyll new myblog

# Change to site directory
cd myblog

# Build and serve locally
bundle exec jekyll serve

# Access at http://localhost:4000
```

#### Method 2: Build Static Site

```bash
# Build site to _site directory
bundle exec jekyll build

# Build with production environment
JEKYLL_ENV=production bundle exec jekyll build
```

#### Method 3: Live Reload Development

```bash
# Serve with live reload
bundle exec jekyll serve --livereload

# Serve with drafts
bundle exec jekyll serve --drafts
```

### CLI Configuration

| Command             | Description                     |
| ------------------- | ------------------------------- |
| `jekyll new <path>` | Create new Jekyll site          |
| `jekyll build`      | Build site to `_site` directory |
| `jekyll serve`      | Build and serve locally         |
| `jekyll clean`      | Remove generated files          |
| `jekyll doctor`     | Check for configuration issues  |

| Serve Options     | Description                          |
| ----------------- | ------------------------------------ |
| `--livereload`    | Reload browser on changes            |
| `--drafts`        | Include draft posts                  |
| `--port <port>`   | Set server port (default: 4000)      |
| `--host <host>`   | Set server host (default: localhost) |
| `--baseurl <url>` | Set base URL                         |

### Security Warning

⚠️ **Jekyll security considerations:**

- Avoid using `safe: false` in production
- Use `exclude` in `_config.yml` to prevent sensitive files from being published
- Sanitize user-generated content if accepting external input
- Keep Jekyll and plugins updated

```yaml
# _config.yml security settings
exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor
```

### Supported Markdown Flavors

| Flavor                   | Support                                |
| ------------------------ | -------------------------------------- |
| Kramdown (default)       | 100%                                   |
| CommonMark               | Via plugin (jekyll-commonmark)         |
| GitHub Flavored Markdown | Via plugin (jekyll-commonmark-ghpages) |
| RedCarpet                | Via plugin (deprecated)                |

Configure markdown processor in `_config.yml`:

```yaml
markdown: kramdown
kramdown:
  input: GFM
  syntax_highlighter: rouge
```

### Troubleshooting

| Issue                    | Solution                           |
| ------------------------ | ---------------------------------- |
| Ruby 3.0+ fails to serve | Run `bundle add webrick`           |
| Gem dependency errors    | Run `bundle install`               |
| Slow builds              | Use `--incremental` flag           |
| Liquid syntax errors     | Check for unescaped `{` in content |
| Plugin not loading       | Add to `_config.yml` plugins list  |

