# AI Tools Interpolation Research: Cursor × Claude Code × GitHub Copilot

> A deep-dive comparison of the three major AI coding configuration ecosystems — what overlaps natively, what is proprietary, and how to design a single workspace that gives all three providers maximum comprehension.

**Date:** April 2026
**Scope:** Cursor (1.7+/2.0), Claude Code (current), GitHub Copilot (CLI + Cloud Agent + VS Code agent mode)
**Audience:** Engineers/teams running more than one AI assistant in the same repo.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Five Configuration Primitives](#2-the-five-configuration-primitives)
3. [Per-Provider Deep Dive](#3-per-provider-deep-dive)
   - [3.1 Cursor](#31-cursor)
   - [3.2 Claude Code](#32-claude-code)
   - [3.3 GitHub Copilot](#33-github-copilot)
4. [Feature-by-Feature Interpolation Matrix](#4-feature-by-feature-interpolation-matrix)
5. [Open Standards: AGENTS.md and SKILL.md](#5-open-standards-agentsmd-and-skillmd)
6. [Cross-Tool Interpolation Map (What "Just Works")](#6-cross-tool-interpolation-map-what-just-works)
7. [Recommended Minimal Workspace Structure](#7-recommended-minimal-workspace-structure)
8. [Anti-patterns and Footguns](#8-anti-patterns-and-footguns)
9. [References](#9-references)

---

## 1. Executive Summary

The three major AI coding tools — Cursor, Claude Code, and GitHub Copilot — have converged on roughly the same five configuration primitives: **memory/instructions**, **rules** (scoped instructions), **skills** (packaged workflows), **agents/modes** (specialized personas), and **hooks** (deterministic lifecycle scripts). However, each tool keeps its own filesystem layout, frontmatter schema, and event names.

Two open standards have emerged that bridge the ecosystem:

- **`AGENTS.md`** — a plain-Markdown "README for agents" stewarded by the Agentic AI Foundation (Linux Foundation), supported natively by Cursor, Copilot CLI, Copilot Cloud Agent, Codex, Gemini CLI, Jules, Amp, Windsurf, and read by Claude Code as a fallback when no `CLAUDE.md` is present.
- **Agent Skills (`SKILL.md`)** — an open spec authored by Anthropic, adopted as-is by GitHub Copilot. Copilot natively reads `.claude/skills/` and `.agents/skills/` in addition to `.github/skills/`.

The practical consequence: **Skills written for Claude Code work in Copilot today**, and **`AGENTS.md` works in essentially every agent except as a primary in Claude Code (which prefers `CLAUDE.md`)**. Hooks and agents/modes remain proprietary per tool.

**TL;DR design rule:** Put your shared, evergreen project context in `AGENTS.md`, put portable workflow playbooks in `.claude/skills/`, and only reach for tool-specific files (`CLAUDE.md`, `.cursor/rules/`, `.github/copilot-instructions.md`, hook configs, custom agents) when you need something that the open formats can't express.

---

## 2. The Five Configuration Primitives

Across the three tools, the same conceptual building blocks recur. Naming differs; behavior differs in detail, but the categories are stable:

| Primitive | What it is | Fires automatically? | Typical scope |
|---|---|---|---|
| **Memory / Instructions** | Always-on guidance loaded at session start | Yes (every prompt) | Project-wide / user-wide |
| **Rules** | Scoped or path-targeted instructions; subset of "memory" with selectivity | Conditionally (by glob, description match, or `alwaysApply`) | Per-folder / per-feature |
| **Skills** | Folder containing `SKILL.md` + scripts/templates the agent loads on demand (progressive disclosure) | Yes — model decides based on `description` frontmatter; or `/skill-name` invocation | Repository or user |
| **Agents / Modes / Subagents** | A specialized persona with its own prompt, tool allowlist, sometimes its own model | No — user picks, or main agent delegates | Repository or user |
| **Hooks** | Deterministic shell scripts/HTTP/LLM evaluators bound to lifecycle events | Yes — fired by the agent loop | Project (committed) and/or user |

Everything else (MCP servers, slash commands, plugins, prompt files, chat modes) reduces to one of these five categories or is a packaging concept on top of them.

---

## 3. Per-Provider Deep Dive

### 3.1 Cursor

Cursor is a fork of VS Code with a deeply integrated agent. As of v2.0 it ships its own model (Composer) and renamed Background Agents to Cloud Agents. Configuration sits across `.cursor/` in the repo, the user-level Cursor Settings, and the deprecated-but-supported root file.

#### Memory / Instructions

Cursor has **three rule tiers** plus **AGENTS.md**:

| Type | Location | Mechanism |
|---|---|---|
| **Project Rules** | `.cursor/rules/*.mdc` (folder, version-controlled) | MDC files with frontmatter (`description`, `globs`, `alwaysApply`) |
| **User Rules** | Cursor Settings → General → Rules for AI | UI-stored, applies across all projects |
| **Team Rules** | Cursor dashboard (Team/Enterprise plans) | Pushed to all members |
| **AGENTS.md** | Repo root (and any subdirectory) | Plain Markdown, "closest wins" |
| **Legacy `.cursorrules`** | Repo root | Deprecated; still loaded for backward compat |

**Key detail on Project Rules:** Each rule lives in `.cursor/rules/<name>/RULE.md` (newer convention) or `.cursor/rules/<name>.mdc` (older), with YAML-style frontmatter:

```yaml
---
description: "RPC service boilerplate"
globs: ["src/services/**/*.ts"]
alwaysApply: false
---
- Use our internal RPC pattern when defining services
- Always use snake_case for service names
@service-template.ts
```

The `@file` syntax pulls additional context into the rule when it triggers. Rules have four invocation styles:
- **Always** — `alwaysApply: true`
- **Auto Attached** — included when files matching `globs` are referenced
- **Agent Requested** — model decides based on `description`
- **Manual** — invoked with `@RuleName`

**Nested rules:** Subdirectories may contain their own `.cursor/rules/` directory, scoped to that folder.

#### Rules (separate from memory)

In Cursor terminology, "Rules" *are* the memory system. There is no second "rules" concept — what Claude Code calls memory and Copilot calls instructions, Cursor calls Rules.

#### Skills

Cursor does **not** have a native Skills feature in its docs. However, the community (and the [Antigravity Awesome Skills](https://github.com/jasonkneen/antigravity-awesome-skills) project) has demonstrated that `SKILL.md` files placed in `.claude/skills/` are read by Cursor's agent because Cursor follows the "closest context wins" rule and will pick up referenced files. This is informal cross-pollination, not first-class.

**Note (Dec 2025):** Cursor has signaled migration toward agent skills as a replacement for some rules use cases.

#### Agents / Modes

Cursor has the most elaborate "modes" system of the three:

| Mode | Description |
|---|---|
| **Agent (default)** | Full tool access, autonomous |
| **Ask** | Q&A, no edits |
| **Manual** | User-driven inline edits |
| **Plan Mode** | Builds a reviewable `plan.md` before code |
| **Custom Modes** | User-defined personas with tool allowlists + custom instructions |
| **Cloud Agents** (formerly Background Agents) | Run remotely on Cursor's AWS infra in isolated VMs, work on a separate branch, push as PR |

**Custom Modes** are configured in-app (Settings → Features → Chat → Custom modes). Cursor has signaled plans to support a `.cursor/modes.json` file but it isn't shipped at the time of this research.

**Cloud Agent setup** lives in `.cursor/environment.json`:

```json
{
  "snapshot": "POPULATED_FROM_SETTINGS",
  "install": "npm install",
  "terminals": [
    { "name": "Run Next.js", "command": "npm run dev" }
  ]
}
```

This file can be committed and is required for Cloud Agents.

#### Hooks

Introduced in **Cursor 1.7 (Sept 2025)**. Configured in JSON, three scopes:

- `.cursor/hooks.json` — project, committed
- `~/.cursor/hooks.json` — user, personal across projects
- Enterprise/admin level

**Lifecycle events (6 total in beta):**

| Event | Fires when |
|---|---|
| `beforeSubmitPrompt` | User submits prompt |
| `beforeShellExecution` | Before any shell command |
| `beforeMCPExecution` | Before MCP tool call |
| `beforeReadFile` | Before agent reads a file |
| `afterFileEdit` | After agent edits a file |
| `stop` | When agent finishes a task |

Hooks receive structured JSON over stdin and return JSON on stdout. Can `permission: "deny"` to block, or modify input. Example:

```json
{
  "version": 1,
  "hooks": {
    "stop": [
      { "command": "osascript -e 'display notification \"Done\"'" }
    ]
  }
}
```

**Important:** Cursor has explicit compatibility with **Claude Code hooks format**, documented under "Third Party Hooks". Read [cursor.com/docs/reference/third-party-hooks](https://cursor.com/docs/reference/third-party-hooks).

#### Other Cursor concepts

- **MCP servers** — `.cursor/mcp.json`
- **Codebase Indexing** — automatic, not configured per-repo
- **`@`-symbols** for context — `@file`, `@folder`, `@web`, `@docs`, `@past-chats`
- **Ignore files** — `.cursorignore` and `.cursorindexingignore`

---

### 3.2 Claude Code

Claude Code is Anthropic's terminal-first agentic coding tool. It is the most opinionated about the filesystem and has the deepest extension surface.

#### Memory / Instructions: `CLAUDE.md`

Claude Code uses a **layered memory hierarchy** of plain Markdown files. Loaded in priority order (later = higher priority):

| Tier | Path | Purpose |
|---|---|---|
| **Managed (policy)** | `/etc/claude-code/CLAUDE.md` | Org-wide, set by admin; cannot be excluded |
| **User** | `~/.claude/CLAUDE.md` | Personal preferences across all projects |
| **Project (ancestors)** | Walks from CWD up to filesystem root, loading every `CLAUDE.md` | Shared with team, version-controlled |
| **Local** | `CLAUDE.local.md` (gitignored) | Personal overrides per project |

CLAUDE.md content is delivered as a **user message after the system prompt**, not as part of the system prompt itself.

**Key features:**
- **`@file` references** — include other files as additional context (max depth 5)
- **Block-level HTML comments** — stripped before injection (`<!-- maintainer notes -->`)
- **`claudeMdExcludes`** setting in `.claude/settings.local.json` to skip ancestor `CLAUDE.md` files in monorepos
- **`/memory`** slash command opens a memory editor and forces reload
- **`#` shorthand** in chat appends a note to the most relevant CLAUDE.md
- **AGENTS.md fallback** — Claude Code does **not** natively load `AGENTS.md`; you must symlink or `@include` it from your `CLAUDE.md`

#### Rules: `.claude/rules/*.md`

A newer subsystem (introduced 2025) for splitting CLAUDE.md content into focused, optionally path-scoped files:

```
.claude/
└── rules/
    ├── testing.md
    ├── typescript-style.md
    └── git-workflow.md
```

Path-scoped frontmatter (Dec 2025+):

```yaml
---
paths:
  - "src/api/**"
  - "src/services/**"
---
Always use dependency injection.
```

This is the closest Claude analogue to Copilot's `applyTo` and Cursor's `globs`.

#### Skills: `.claude/skills/`

**The flagship Claude Code feature for portable, reusable workflows.** Skills are folders, each with a `SKILL.md`:

```
.claude/skills/
└── pdf-processing/
    ├── SKILL.md          # Required: name, description, body
    ├── FORMS.md          # Optional reference material
    ├── validate_form.py  # Optional executable script
    └── templates/
```

`SKILL.md` frontmatter:

```yaml
---
name: pdf-processing
description: Extract text and tables from PDFs, fill forms, merge documents. Use when working with PDF files or when the user mentions PDFs, forms, or document extraction.
---
```

**Progressive disclosure** is the architectural innovation:
1. **Startup**: Only `name` + `description` are loaded into context (~100 chars per skill).
2. **Triggered**: When the description matches the request, Claude reads `SKILL.md` body via bash.
3. **On-demand**: Referenced files (e.g., `FORMS.md`, scripts) are read only when needed.

Skill scopes:
- `~/.claude/skills/` — user-level, all projects
- `.claude/skills/` — project-level, committed
- Plugins can bundle skills in `skills/` directory
- Managed (org-wide via policy)

Skills can also be **invoked explicitly** as slash commands: `/pdf-processing`. The legacy `.claude/commands/<name>.md` format still works and is essentially a flat skill.

**Skills are an open standard** and work on Claude.ai, Claude Desktop, the API, and Claude Code identically.

#### Subagents: `.claude/agents/`

Specialized agents with their own context window, system prompt, and tool allowlist. Files have YAML frontmatter:

```yaml
---
name: security-reviewer
description: Reviews code changes for security vulnerabilities. Use proactively before commits touching auth, payments, or user data.
tools: Read, Grep, Glob
model: sonnet
---
You are a security-focused code reviewer. Analyze the provided changes for:
- SQL injection, XSS, and command injection risks
- Authentication and authorization gaps
...
```

**Built-in subagents:** `Explore`, `Plan`, `general-purpose`. Custom subagents live at `.claude/agents/` (project) or `~/.claude/agents/` (user). Project takes precedence on name conflicts.

The main agent **delegates automatically** based on the subagent's `description` field, or you call them with `/agents` interactively. Subagents preserve main-context budget by isolating noisy work in their own session.

**Distinction from Skills:** Subagents *replace* the runtime; Skills *extend* the same agent's playbook.

#### Hooks: `.claude/settings.json`

The most sophisticated hooks system of the three. Configured in `settings.json` at project, user, or managed level.

**~21 lifecycle events** (as of March 2026):

`PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `Stop`, `SubagentStop`, `Notification`, `SessionStart`, `SessionEnd`, `PreCompact`, `PostCompact`, `UserPromptSubmit`, `InstructionsLoaded`, `ConfigChange`, `WorktreeCreate`, `WorktreeRemove`, `Elicitation`, `ElicitationResult`, and more.

**Four handler types:**

| Type | Purpose |
|---|---|
| `command` | Shell script (most common) |
| `http` | POST to an endpoint, receive JSON back (Feb 2026+) |
| `prompt` | Send a text prompt to a fast model (Haiku) for semantic eval |
| `agent` | Spawn a subagent with tool access for deep verification |

Async hooks (`async: true`) added Jan 2026 — non-blocking.

**Example — auto-format on edit:**

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          { "type": "command", "command": "npx prettier --write \"$CLAUDE_TOOL_INPUT_FILE_PATH\"" }
        ]
      }
    ]
  }
}
```

Exit code 2 from a `PreToolUse` hook **denies** the tool call. JSON output can return `{ "permissionDecision": "deny", "additionalContext": "..." }` for richer control.

#### Plugins

Plugins are git repositories bundling skills, subagents, commands, hooks, and MCP servers. Installed via `/plugin install <name>@<marketplace>`. Structure:

```
plugin-name/
├── .claude-plugin/
│   └── plugin.json
├── commands/
├── agents/
├── skills/
├── hooks/
├── .mcp.json
└── README.md
```

#### Settings file: `.claude/settings.json`

The unified config. Keys include `permissions`, `hooks`, `claudeMdExcludes`, `env`, `model`, `mcpServers`, and more. Variants: `settings.json` (project, committed), `settings.local.json` (gitignored), `~/.claude/settings.json` (user), and managed policy.

---

### 3.3 GitHub Copilot

Copilot has the broadest IDE reach (VS Code, Visual Studio, JetBrains, Eclipse, Xcode, Neovim, Zed, SSMS) and three distinct surfaces with **subtly different config support**:

- **Copilot in IDEs** (Chat + agent mode + completions)
- **Copilot CLI** (`copilot` terminal tool)
- **Copilot Cloud Agent** (formerly Coding Agent — runs PRs autonomously on github.com)

#### Memory / Instructions: `.github/copilot-instructions.md` + `AGENTS.md`

Copilot supports **three levels** of instructions:

| Level | File | Scope |
|---|---|---|
| **Personal** | UI-managed at github.com/copilot or VS Code settings | Per-user across repos |
| **Repository** | `.github/copilot-instructions.md` | Repo-wide, committed |
| **Organization** | Set in org settings | All repos in the org |

**Priority:** Personal > Repository > Organization. All three are concatenated when relevant.

**Path-specific instructions** live in `.github/instructions/<name>.instructions.md` with `applyTo` frontmatter:

```yaml
---
applyTo: "**/*.tsx"
---
Use functional components with TypeScript interfaces.
Always use server components unless client interactivity is required.
```

**`AGENTS.md` is natively recognized** by:
- Copilot Cloud Agent
- Copilot CLI
- Copilot in VS Code (when in agent mode)

When **both** `AGENTS.md` and `.github/copilot-instructions.md` exist at the root, **both are loaded**. Additional `AGENTS.md` files in subdirectories or in `COPILOT_CUSTOM_INSTRUCTIONS_DIRS` are treated as supplementary.

**Copilot CLI also reads `CLAUDE.md` and `GEMINI.md` as fallbacks** when in the repo root. This is critical for cross-tool design.

#### Rules

Copilot doesn't have a separate "Rules" concept; what others call rules, Copilot calls **path-specific instructions** (`.github/instructions/*.instructions.md`).

#### Skills: `.github/skills/` + `.claude/skills/` + `.agents/skills/`

**Released Dec 2025.** Copilot Agent Skills follow the **same open spec as Claude's**. Copilot natively reads:

- `.github/skills/<name>/SKILL.md` (project, committed)
- `.claude/skills/<name>/SKILL.md` (project, committed) — **direct Claude skill compatibility**
- `.agents/skills/<name>/SKILL.md` (project, committed) — neutral path
- `~/.copilot/skills/<name>/SKILL.md` (user)
- `~/.claude/skills/<name>/SKILL.md` (user) — **direct Claude skill compatibility**
- `~/.agents/skills/<name>/SKILL.md` (user)

This means: **if you have skills in `.claude/skills/`, Copilot picks them up automatically. No symlink required.**

Copilot's skill discovery, progressive loading, and `/skill-name` invocation behave identically to Claude Code's. Some Copilot-specific frontmatter properties:
- `allowed-tools` — restricts what tools the skill can call
- `user-invocable` — controls whether the skill appears in the slash menu
- `disable-model-invocation` — prevents auto-invocation

Available in Copilot Cloud Agent, Copilot CLI, and VS Code agent mode (with `chat.useAgentSkills` enabled).

#### Custom Agents: `.github/agents/<name>.agent.md` (was `.chatmode.md`)

**Renamed from "Chat Modes" to "Custom Agents" in late 2025.** The functionality is the same.

Locations:
- **Repository** — `.github/agents/<name>.agent.md`
- **Organization/Enterprise** — `/agents/<name>.agent.md` in a `.github-private` repo
- **User (VS Code)** — VS Code user profile
- **User (CLI)** — `~/.config/copilot/agents/<name>.agent.md`

Frontmatter:

```yaml
---
description: Generate an implementation plan for new features or refactoring existing code
tools: ['search', 'web']
model: GPT-5.2
handoffs:
  - label: Start Implementation
    agent: implementation
    prompt: Now implement the plan above
    send: false
---
```

Notable features:
- **`handoffs`** — buttons that switch the user to another agent with a pre-filled prompt
- **`tools` allowlist** — restrict what the agent can do
- **`mcp-servers`** — agent-specific MCP servers
- **`runSubagent` tool** — Copilot agents can spawn subagents (similar to Claude's pattern)

**Built-in agents in VS Code:** Default `Agent`, `Plan`, `Edit` (and Visual Studio adds `@debugger`, `@profiler`, `@test`, `@modernize`).

**Equivalence:** Copilot Custom Agents ≈ Cursor Custom Modes ≈ Claude Code Subagents (all configure persona + tools + sometimes model).

#### Hooks: `.github/hooks/*.json`

**Released ~Q4 2025.** Repository-bound by design (single source of truth for the team via git).

**6 lifecycle events** (Cloud Agent + CLI):

| Event | Notes |
|---|---|
| `sessionStart` | New or resumed session |
| `sessionEnd` | Session terminating |
| `userPromptSubmitted` | Prompt sent |
| `preToolUse` | **Can deny tool execution via JSON output** |
| `postToolUse` | After tool runs |
| `errorOccurred` | Tool/agent error |

VS Code adds two more (`agentStop`, `subagentStop`) and supports **8 events** total.

Configuration format (JSON, supports both Bash and PowerShell):

```json
{
  "version": 1,
  "hooks": {
    "preToolUse": [
      {
        "type": "command",
        "bash": "./.github/hooks/security-check.sh",
        "powershell": "./.github/hooks/security-check.ps1",
        "timeoutSec": 30
      }
    ]
  }
}
```

**Cross-tool note:** VS Code's hook implementation **also reads `.claude/settings.json`, `.claude/settings.local.json`, and `~/.claude/settings.json`** by default — it parses Claude Code's hook configuration format and converts event names from camelCase to PascalCase. This means **a Claude Code hooks config will fire in Copilot's VS Code agent mode** (with caveats: matchers like `Edit|Write` are parsed but not applied — all hooks run on every event).

#### Prompt Files & Chat Customizations

`.github/prompts/<name>.prompt.md` — reusable prompts invoked via `/prompt-name`. Available in VS Code, Visual Studio, JetBrains. These are **not** standard slash commands; they are templated prompts with optional `${input:varname}` placeholders.

Example:
```yaml
---
agent: 'agent'
description: 'Generate a clear code explanation with examples'
---
Explain the following code in a clear, beginner-friendly way:
Code to explain: ${input:code:Paste your code here}
```

#### MCP Servers

Configured in `.vscode/mcp.json` (workspace), `~/.config/copilot/mcp.json` (user), or in agent profiles.

---

## 4. Feature-by-Feature Interpolation Matrix

This is the heart of the report. **N** = native, **P** = portable via open standard, **S** = supported via symlink/manual mapping, **✗** = not supported.

| Feature | Cursor | Claude Code | GitHub Copilot |
|---|---|---|---|
| **Project memory file** | `AGENTS.md` (N), `.cursorrules` (legacy) | `CLAUDE.md` (N), `AGENTS.md` (S — must `@include` or symlink) | `.github/copilot-instructions.md` (N), `AGENTS.md` (N), `CLAUDE.md` (N — CLI only), `GEMINI.md` (N — CLI only) |
| **User-level memory** | UI-stored (User Rules) | `~/.claude/CLAUDE.md` (N) | UI-stored personal instructions; `%USERPROFILE%/copilot-instructions.md` in some IDEs |
| **Path-scoped rules** | `.cursor/rules/*.mdc` with `globs` (N) | `.claude/rules/*.md` with `paths` frontmatter (N, Dec 2025+) | `.github/instructions/*.instructions.md` with `applyTo` (N) |
| **Nested directory rules** | Subdirectory `.cursor/rules/` (N), nested `AGENTS.md` (N) | Ancestor `CLAUDE.md` walk (N), nested `AGENTS.md` (✗ unless wired in) | Nested `AGENTS.md` (N for CLI/Cloud), `COPILOT_CUSTOM_INSTRUCTIONS_DIRS` env var |
| **Skills (`SKILL.md`)** | ✗ native (P — informal pickup of `.claude/skills/`) | `.claude/skills/` (N), `~/.claude/skills/` (N) | `.github/skills/` (N), **`.claude/skills/` (N)**, `.agents/skills/` (N), `~/.claude/skills/` (N) |
| **Specialized agents** | Custom Modes (N, in-app config); Cloud Agents (N, `.cursor/environment.json`) | Subagents in `.claude/agents/*.md` (N) | Custom Agents in `.github/agents/*.agent.md` (N) |
| **Built-in agents** | Agent / Ask / Manual / Plan | Explore / Plan / general-purpose | Agent / Plan / Edit (+ IDE-specific) |
| **Subagent / parallel delegation** | Subagents in Cloud Agent (N) | Subagents (N), Agent Teams (N — separate sessions) | `runSubagent` tool, custom agent delegation (N) |
| **Hooks lifecycle events** | 6 events (`beforeSubmitPrompt`, `beforeShellExecution`, `beforeMCPExecution`, `beforeReadFile`, `afterFileEdit`, `stop`) | 21+ events (`PreToolUse`, `PostToolUse`, `Stop`, `SessionStart`, etc.) | 6 events in CLI/Cloud (`sessionStart`, `sessionEnd`, `userPromptSubmitted`, `preToolUse`, `postToolUse`, `errorOccurred`); 8 in VS Code |
| **Hooks config location** | `.cursor/hooks.json` (project), `~/.cursor/hooks.json` (user) | `.claude/settings.json` (project), `~/.claude/settings.json` (user), managed | `.github/hooks/*.json` (project, branch-default required for Cloud Agent); CLI loads from CWD |
| **Hooks handler types** | command (script) | command, http, prompt, agent (LLM-evaluated) | command (Bash + PowerShell) |
| **Hooks cross-tool reading** | Documented Claude Code hook format compatibility | n/a | VS Code reads `.claude/settings.json` natively |
| **Hooks: deny capability** | `permission: "deny"` JSON output | Exit 2 or `permissionDecision: "deny"` | `preToolUse` JSON `permissionDecision` |
| **MCP servers** | `.cursor/mcp.json` | `.mcp.json` or `.claude/settings.json` | `.vscode/mcp.json` or in agent profile |
| **Slash / prompt commands** | `/` chat commands; some custom modes act as commands | `.claude/commands/*.md` (legacy), now merged into Skills | `.github/prompts/*.prompt.md` |
| **Plugin / marketplace system** | Limited (community modes) | Plugins (`/plugin install`) bundling skills/agents/hooks/MCP | Plugins via `copilot plugin install` (similar bundling) |
| **Ignore files** | `.cursorignore`, `.cursorindexingignore` | Standard `.gitignore` honored | `.copilotignore` and standard `.gitignore` |

### Key takeaways from the matrix

1. **Skills are the most portable primitive.** A `.claude/skills/<skill>/SKILL.md` works in Claude Code natively, in Copilot natively (Copilot reads `.claude/skills/`), and is informally picked up by Cursor.
2. **`AGENTS.md` is the most portable memory format.** It works natively in Cursor and Copilot. Claude Code does not auto-load it but you can `@AGENTS.md` from `CLAUDE.md` (or symlink).
3. **Path-scoped rules use 3 different frontmatter keys** for the same idea: Cursor `globs`, Claude `paths`, Copilot `applyTo`. Not portable.
4. **Hooks are essentially proprietary.** Cursor explicitly shipped Claude Code hook compatibility, and VS Code's Copilot Chat hook implementation reads Claude's settings.json — but Copilot Cloud Agent and CLI use their own format.
5. **Agents/modes are entirely proprietary.** Each tool defines its persona/mode files differently. No cross-tool standard exists yet.

---

## 5. Open Standards: AGENTS.md and SKILL.md

These two formats are the only true cross-tool primitives in 2026.

### `AGENTS.md`

**Stewarded by:** the Agentic AI Foundation (AAIF), Linux Foundation.
**Contributors:** OpenAI Codex, Cursor, Amp, Jules (Google), Factory, Sourcegraph, GitHub Copilot.
**Adopted by:** 40,000+ open-source projects (as of late 2025).

**Spec, in essence:**
- Plain Markdown, no required schema
- Place at repo root and (optionally) in subdirectories
- Closest `AGENTS.md` to the file being edited wins
- Explicit user prompts override everything

**What works where:**

| Tool | Reads root `AGENTS.md`? | Reads nested `AGENTS.md`? |
|---|---|---|
| Cursor | ✅ Native | ✅ Native (combined with parents, more specific wins) |
| Copilot Cloud Agent | ✅ Native (alongside `.github/copilot-instructions.md`) | ✅ Via `COPILOT_CUSTOM_INSTRUCTIONS_DIRS` |
| Copilot CLI | ✅ Native (also reads `CLAUDE.md` and `GEMINI.md` as fallbacks) | ✅ |
| Copilot in VS Code | ✅ Native | ✅ |
| Claude Code | ❌ Not auto-loaded | ❌ — must `@AGENTS.md` from CLAUDE.md or symlink |
| Codex CLI | ✅ Primary file | ✅ |
| Gemini CLI | Reads `GEMINI.md`; symlink AGENTS.md | – |

### Agent Skills (`SKILL.md`)

**Authored by:** Anthropic.
**Adopted by:** Claude (all surfaces — Code, Desktop, Web, API), GitHub Copilot (Cloud Agent, CLI, VS Code), and increasingly community-built tooling around Cursor and Codex.

**Spec, in essence:**
- A folder containing a required `SKILL.md` with YAML frontmatter (`name`, `description`)
- Markdown body = the playbook the agent reads when triggered
- Optional bundled scripts, references, templates
- **Progressive disclosure:** only the description loads at startup; the body loads when triggered; bundled files load on demand

**Frontmatter validation rules:**
- `name`: lowercase, hyphens-for-spaces, ≤64 chars, no reserved words ("anthropic", "claude")
- `description`: ≤1024 chars
- Combined entry text is capped at ~1,536 characters in many implementations

**Discovery paths (Copilot's official list, which Claude Code also honors):**

```
Project (committed):  .github/skills/  |  .claude/skills/  |  .agents/skills/
User (personal):      ~/.copilot/skills/  |  ~/.claude/skills/  |  ~/.agents/skills/
```

**Practical recommendation:** Use `.claude/skills/` as the canonical project location. It is read by Claude Code natively and by Copilot natively. This single placement gives you two-tool coverage with zero symlinks.

---

## 6. Cross-Tool Interpolation Map (What "Just Works")

Concrete answer to the prompt's two motivating examples — and many more.

### What flows freely (no extra work)

| If you put... | ...it works in |
|---|---|
| `AGENTS.md` (root) | Cursor ✅, Copilot ✅ (Cloud + CLI + VS Code), Codex ✅, Gemini (via symlink) — **Claude Code: only via `@AGENTS.md` in CLAUDE.md** |
| `.claude/skills/<skill>/SKILL.md` | Claude Code ✅, Copilot ✅ (Cloud + CLI + VS Code), Cursor (informal context pickup) |
| `~/.claude/skills/<skill>/SKILL.md` | Claude Code ✅, Copilot ✅ |
| `CLAUDE.md` (root) | Claude Code ✅, **Copilot CLI ✅ (read as fallback)** |
| `.claude/settings.json` (hooks) | Claude Code ✅, Copilot in VS Code ✅ (read natively, with matcher caveat) |
| `.claude/agents/<agent>.md` (subagent) | Claude Code ✅ — others ignore |
| `.cursor/rules/*.mdc` | Cursor only |
| `.cursor/hooks.json` | Cursor only |
| `.cursor/environment.json` | Cursor Cloud Agent only |
| `.github/copilot-instructions.md` | Copilot only |
| `.github/instructions/*.instructions.md` | Copilot only |
| `.github/agents/*.agent.md` | Copilot only |
| `.github/hooks/*.json` | Copilot only (CLI requires file in CWD; Cloud requires merge to default branch) |
| `.github/prompts/*.prompt.md` | Copilot in VS Code/JetBrains only |

### Symlink strategies (when you need a single source of truth)

The canonical pattern (battle-tested across the ecosystem):

```bash
# AGENTS.md is the source of truth for shared instructions
# Hard to symlink Markdown into Claude Code's native loader because Claude won't auto-load AGENTS.md,
# so we go the other direction: CLAUDE.md → AGENTS.md
ln -s AGENTS.md CLAUDE.md

# Copilot already reads AGENTS.md natively in Cloud + CLI; for VS Code IDE, optionally:
ln -s ../AGENTS.md .github/copilot-instructions.md

# For Cursor's Project Rules system (when you want AGENTS.md content available as a referenced rule):
mkdir -p .cursor/rules
ln -s ../../AGENTS.md .cursor/rules/agents.md
```

A more conservative pattern: keep `AGENTS.md` as 80% shared content, and use tool-specific files only for the 20% that's truly tool-specific (e.g., MCP settings in `CLAUDE.md`, `applyTo` patterns in Copilot path-instructions).

---

## 7. Recommended Minimal Workspace Structure

A repository structure that maximizes coverage across all three tools with **minimal duplication** and **no symlinks** required for the most-impactful primitives:

```
your-project/
│
├── AGENTS.md                          # Shared, evergreen project context.
│                                       # Read natively by Cursor + Copilot (all surfaces).
│                                       # Referenced from CLAUDE.md (see below).
│
├── CLAUDE.md                          # Claude-specific overlay.
│                                       # First line: "@AGENTS.md" to inherit shared content.
│                                       # Then add: model preferences, MCP notes, Anthropic-specific quirks.
│
├── .claude/
│   ├── settings.json                  # Hooks (Claude + Copilot VS Code).
│   ├── settings.local.json            # Personal overrides (gitignored).
│   ├── rules/                         # Optional: split CLAUDE.md by topic.
│   │   ├── testing.md
│   │   └── conventions.md
│   ├── skills/                        # Skills — read by Claude AND Copilot natively.
│   │   ├── pdf-processing/
│   │   │   └── SKILL.md
│   │   └── code-review/
│   │       └── SKILL.md
│   ├── agents/                        # Subagents (Claude only).
│   │   └── security-reviewer.md
│   └── commands/                      # Legacy slash commands (still work).
│
├── .cursor/
│   ├── rules/                         # Cursor-specific path-scoped rules.
│   │   └── frontend.mdc
│   ├── hooks.json                     # Cursor lifecycle hooks.
│   ├── mcp.json                       # Cursor MCP servers.
│   └── environment.json               # Cursor Cloud Agent setup.
│
├── .github/
│   ├── copilot-instructions.md        # Optional: Copilot-specific overlay.
│   │                                   # Often unnecessary if AGENTS.md covers it.
│   ├── instructions/                  # Path-specific Copilot rules with applyTo.
│   │   └── react.instructions.md
│   ├── agents/                        # Copilot custom agents.
│   │   └── refactor-specialist.agent.md
│   ├── hooks/                         # Copilot Cloud Agent + CLI hooks.
│   │   └── security-policy.json
│   └── prompts/                       # Copilot reusable prompts (VS Code).
│       └── explain-code.prompt.md
│
├── .vscode/
│   └── mcp.json                       # Copilot in VS Code MCP config.
│
└── (rest of repo)
```

### User-level companion

```
~/
├── .claude/
│   ├── CLAUDE.md                       # Personal preferences across all projects.
│   ├── settings.json                   # Personal hooks.
│   └── skills/                         # Personal skills (also picked up by Copilot).
│
├── .cursor/
│   └── hooks.json                      # Personal Cursor hooks.
│
└── .config/copilot/
    ├── agents/                          # Personal Copilot CLI custom agents.
    └── mcp.json                         # Personal Copilot CLI MCP config.
```

### Minimal CLAUDE.md template that maximizes interop

```markdown
@AGENTS.md

# Claude-specific notes
- Default thinking model: opus when planning, sonnet when implementing
- Hooks: see .claude/settings.json
- For repo-wide conventions, see AGENTS.md (loaded above)
```

### Skill placement decision tree

```
Want this skill in Claude Code only?      → ~/.claude/skills/<name>/
Want it in Claude + Copilot, project-scoped? → .claude/skills/<name>/   (best default)
Want it in Copilot only, project-scoped?     → .github/skills/<name>/
Want it portable to all skill-aware agents?  → .agents/skills/<name>/   (most neutral)
```

---

## 8. Anti-patterns and Footguns

### Duplication (the most common mistake)

Copying the same rules into `CLAUDE.md`, `.cursorrules`, and `.github/copilot-instructions.md` creates **three sources of truth that drift over time**. Use `AGENTS.md` + `@AGENTS.md` references, or symlinks.

### Trusting auto-generators (`/init`)

Both Claude Code and Copilot can auto-generate their instruction files. The output is generic, bloated, and full of conventions a linter already enforces. **Hand-write your AGENTS.md.** Every line should earn its place.

### Including linter-handled rules

"Use 2-space indentation," "always add trailing commas" — these are Prettier's job. Don't burn agent context budget on them.

### Writing skill descriptions that don't trigger

Skills load only when the agent matches the user request to the skill's `description`. If your description is vague (`"Helps with PDFs"`), the agent won't reach for the skill. Front-load the use cases: `"Extract text and tables from PDF files, fill forms, merge documents. Use when working with PDF files or when the user mentions PDFs, forms, or document extraction."`

### Hooks that block on every event

A `Stop` hook that blocks Claude indefinitely creates an infinite loop. A `preToolUse` hook with broad denial blocks normal work. Always test hooks with explicit narrow matchers and easy bypass paths.

### Forgetting Cloud Agent merge requirements

Copilot Cloud Agent loads `.github/hooks/*.json` only from the **default branch**. A hook on a feature branch doesn't run. Cursor Cloud Agents require `.cursor/environment.json` to be committed.

### Trusting `applyTo`/`globs`/`paths` matching

These are LLM-driven matchers in some implementations and exact glob matches in others. For Cursor and Copilot they're glob-based; for Claude rules they're glob-based on path; for matchers in Claude Code hooks, the matcher applies to **tool names**, not files. Read the docs per tool.

### Skill name collisions

A skill named `format-code` in `.claude/skills/` and one in `~/.claude/skills/` and one in a plugin all create `/format-code`. Project takes precedence in most tools but not all. Namespace them: `team-format-code`, `personal-format-code`.

### Symlinking AGENTS.md into Claude Code's CLAUDE.md path on Windows

Windows symlink semantics for Markdown loaders are flaky. Prefer `@AGENTS.md` reference syntax in `CLAUDE.md`, which works cross-platform and is explicit about what's being included.

### Treating subagents/custom-agents/custom-modes as portable

They aren't. A Claude `.claude/agents/security-reviewer.md` is invisible to Cursor, and a Copilot `.github/agents/security-reviewer.agent.md` is invisible to Claude Code. Some symlink approaches are documented (e.g., `.claude/agents -> ../.github/agents`) but the **frontmatter schemas differ** (Claude: `name, description, tools, model`; Copilot: `description, tools, model, handoffs, mcp-servers`), and tool name aliases differ across platforms. Treat agent definitions as tool-specific.

---

## 9. References

### Cursor
- Cursor Docs hub — https://cursor.com/docs
- Rules — https://cursor.com/docs/rules and https://docs.cursor.com/context/rules
- Hooks — https://cursor.com/docs/hooks
- Third-Party (Claude Code) Hooks Compatibility — https://cursor.com/docs/reference/third-party-hooks
- Custom Modes — https://docs.cursor.com/chat/custom-modes
- Background/Cloud Agents — https://docs.cursor.com/en/background-agent
- Cursor 1.7 Changelog (Hooks GA) — https://cursor.com/changelog/1-7
- Cursor 2.0 Changelog (Composer + Cloud Agents rename) — https://cursor.com/changelog/2-0

### Claude Code
- Claude Code Docs hub — https://code.claude.com/docs/en/
- Skills — https://code.claude.com/docs/en/skills
- Subagents — https://code.claude.com/docs/en/sub-agents
- Hooks Guide — https://code.claude.com/docs/en/hooks-guide
- Memory (CLAUDE.md) — https://code.claude.com/docs/en/memory
- Plugins — https://github.com/anthropics/claude-code/blob/main/plugins/README.md
- Agent Skills (API/spec) — https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview
- Public skills repo — https://github.com/anthropics/skills

### GitHub Copilot
- Copilot Docs hub — https://docs.github.com/en/copilot
- Custom Instructions — https://docs.github.com/en/copilot/how-tos/configure-custom-instructions
- Repository instructions — https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot
- Path-specific instructions support matrix — https://docs.github.com/en/copilot/reference/custom-instructions-support
- About Agent Skills — https://docs.github.com/en/copilot/concepts/agents/about-agent-skills
- Adding Agent Skills (Cloud) — https://docs.github.com/en/copilot/how-tos/use-copilot-agents/cloud-agent/add-skills
- Adding Agent Skills (CLI) — https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-skills
- Custom Agents (Cloud) — https://docs.github.com/en/copilot/how-tos/use-copilot-agents/cloud-agent/create-custom-agents
- Custom Agents Configuration — https://docs.github.com/en/copilot/reference/custom-agents-configuration
- Custom Agents (CLI) — https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/create-custom-agents-for-cli
- Hooks Configuration — https://docs.github.com/en/copilot/reference/hooks-configuration
- Using Hooks (CLI) — https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks
- VS Code Custom Agents — https://code.visualstudio.com/docs/copilot/customization/custom-agents
- VS Code Agent Skills — https://code.visualstudio.com/docs/copilot/customization/agent-skills
- VS Code Hooks (with Claude Code interop) — https://code.visualstudio.com/docs/copilot/customization/hooks
- VS Code Custom Instructions — https://code.visualstudio.com/docs/copilot/customization/custom-instructions
- Awesome Copilot — https://github.com/github/awesome-copilot

### Open Standards
- AGENTS.md spec & adopters — https://agents.md/
- Agent Skills (Anthropic-published, used by Copilot) — https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview

### Comparison & community
- "How to Configure Every AI Coding Assistant" — DeployHQ, Mar 2026
- "AGENTS.md Cross-Tool Unified Management Guide" — SmartScope, Feb 2026
- "Tips for Coexisting GitHub Copilot Settings with Claude Code and Other AI Tools" — zenn.dev/kesin11, Dec 2025
- GitHub Changelog: Copilot Agent Skills support — Dec 18, 2025

---

*End of research document.*