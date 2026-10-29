# Skill link topology

Where installed skills come from and how to verify their source mapping.
Infrastructure reference; needed when adding or tracing a skill, not during a
code change.

- Shared local skills are embedded from `static/__shared__/skills` and installed under `~/.claude/skills/<skill-id>/`.
- External skills are declared in `static/external-skills.toml` and installed under `~/.claude/skills/<source-id>/`.
- Docker skills are sourced externally as individual ids (for example `docker-core-architecture`) rather than grouped static folders.
- Validate source mapping with: `grep '^id = "' static/external-skills.toml`.
