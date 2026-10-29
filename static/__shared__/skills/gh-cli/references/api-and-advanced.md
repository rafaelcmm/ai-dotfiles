# API, Rulesets, and Advanced Commands

`gh api`, `gh ruleset`, `gh attestation`, `gh completion`, `gh preview`, and
`gh agent-task` command reference.

## Contents

- [API Requests (gh api)](#api-requests-gh-api)
- [Rulesets (gh ruleset)](#rulesets-gh-ruleset)
- [Attestations (gh attestation)](#attestations-gh-attestation)
- [Completion (gh completion)](#completion-gh-completion)
- [Preview (gh preview)](#preview-gh-preview)
- [Agent Tasks (gh agent-task)](#agent-tasks-gh-agenttask)

## API Requests (gh api)

```bash
# Make API request
gh api /user

# Request with method
gh api --method POST /repos/owner/repo/issues \
  --field title="Issue title" \
  --field body="Issue body"

# Request with headers
gh api /user \
  --header "Accept: application/vnd.github.v3+json"

# Request with pagination
gh api /user/repos --paginate

# Raw output (no formatting)
gh api /user --raw

# Include headers in output
gh api /user --include

# Silent mode (no progress output)
gh api /user --silent

# Input from file
gh api --input request.json

# jq query on response
gh api /user --jq '.login'

# Field from response
gh api /repos/owner/repo --jq '.stargazers_count'

# GitHub Enterprise
gh api /user --hostname enterprise.internal

# GraphQL query
gh api graphql \
  -f query='
  {
    viewer {
      login
      repositories(first: 5) {
        nodes {
          name
        }
      }
    }
  }'
```

## Rulesets (gh ruleset)

```bash
# List rulesets
gh ruleset list

# View ruleset
gh ruleset view 123

# Check ruleset
gh ruleset check --branch feature

# Check specific repository
gh ruleset check --repo owner/repo --branch main
```

## Attestations (gh attestation)

```bash
# Download attestation
gh attestation download owner/repo \
  --artifact-id 123456

# Verify attestation
gh attestation verify owner/repo

# Get trusted root
gh attestation trusted-root
```

## Completion (gh completion)

```bash
# Generate shell completion
gh completion -s bash > ~/.gh-complete.bash
gh completion -s zsh > ~/.gh-complete.zsh
gh completion -s fish > ~/.gh-complete.fish
gh completion -s powershell > ~/.gh-complete.ps1

# Shell-specific instructions
gh completion --shell=bash
gh completion --shell=zsh
```

## Preview (gh preview)

```bash
# List preview features
gh preview

# Run preview script
gh preview prompter
```

## Agent Tasks (gh agent-task)

```bash
# List agent tasks
gh agent-task list

# View agent task
gh agent-task view 123

# Create agent task
gh agent-task create --description "My task"
```

