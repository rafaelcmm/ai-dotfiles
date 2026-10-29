# Third-party content

Some skills in this directory are adapted from third-party work. This file records
what, from where, and under what terms.

## obra/superpowers — MIT

Source: <https://github.com/obra/superpowers>
Studied at commit `2ce8aa15d05bd57213e6f746f953aafb7ded7a8b`.

Companion repositories read at the same time:

| Repository | Commit |
| --- | --- |
| `obra/superpowers-skills` | `cdcd624ad3fd8026deb692e565351854569798dd` |
| `obra/superpowers-lab` | `51111f74f24058117752d9aa917cb19859f8ec86` |

The `superpowers-skills` copies were materially older than the plugin repository's own
and in one case carried inverted advice. Material here is adapted from the plugin
repository unless a skill says otherwise.

### Skills adapted substantially

Structure, rules, or phrasing derive directly from upstream. Each carries its own
`## Attribution` section.

| Skill | Upstream origin |
| --- | --- |
| `writing-skills` | `skills/writing-skills` + its references |
| `auditing-skill-libraries` | `skills/meta/gardening-skills-wiki` (scripts rewritten from scratch) |
| `subagent-dispatch` | `skills/subagent-driven-development`, `skills/dispatching-parallel-agents` |
| `execution-ledger` | `skills/executing-plans` |
| `review-fix-loop` | `skills/subagent-driven-development/references/fix-loop.md` |
| `model-tier-economics` | the cost design specs under `docs/superpowers/specs/` |
| `receiving-code-review` | `skills/collaboration/receiving-code-review` |
| `verification-before-completion` | `skills/verification-before-completion` |
| `writing-good-tests` | `skills/test-driven-development/writing-good-tests.md` |
| `systematic-debugging` | `skills/systematic-debugging` + its references |
| `session-self-audit` | `skills/diagnosing-superpowers` (issue filing and share-bundle export removed) |
| `when-stuck` | `skills/problem-solving/*`, `skills/architecture/preserving-productive-tensions` |
| `pressure-testing-instructions` | the `systematic-debugging` pressure tests |
| `driving-interactive-commands` | `superpowers-lab/skills/using-tmux-for-interactive-commands` |

### Ideas and findings used without substantial copying

`code-review` and its references, `code-change-workflow`, and `commenting-standards`
take specific disciplines and measured findings from the same source and credit it
inline where it matters.

### Measured figures

Cost, token, timing and defect-detection numbers quoted in these skills are **upstream's
experimental results**, cited as evidence. They were not reproduced here and must not be
presented as our own measurements.

### Not adopted

Nothing that posts, publishes, pushes, opens or merges a pull request, submits a review,
replies in a review thread, comments on a ticket, or files an issue was carried over. See
`.claude/rules/acting-in-the-users-name.md` for the standing constraint that governs this.

### MIT License

```
MIT License

Copyright (c) 2025 Jesse Vincent

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Microsoft Amplifier

Source: <https://github.com/microsoft/amplifier>, commit
`2adb63f858e7d760e188197c8e8d4c1ef721e2a6` (2025-10-10).

The lateral-thinking techniques in `when-stuck` originate here. `obra/superpowers`
adapted them into skills, and that adaptation is what this repository worked from, so
both are credited. Upstream credits Amplifier in its `research/`, `problem-solving/` and
`architecture/` categories only — no other skill in this directory derives from it.

## Anthropic

Source: Anthropic's published agent-skills documentation.

The skill-authoring guidance on degrees of freedom matched to task fragility, treating
the context window as a public good, tables of contents on reference files over 100
lines, and keeping references one level deep is Anthropic's own. `obra/superpowers`
vendors a copy of that document; the guidance is credited to Anthropic rather than to the
repository that vendored it.

## Skills that are not third-party

`revisao-texto-ptbr` is the maintainer's own prior work, generalized for reuse. Its
reference files carry their own inline citations for the linguistic sources they draw on.
