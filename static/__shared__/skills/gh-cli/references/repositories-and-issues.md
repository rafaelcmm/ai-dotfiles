# Repositories and Issues

`gh browse`, `gh repo`, and `gh issue` command reference.

## Contents

- [Browse (gh browse)](#browse-gh-browse)
- [Repositories (gh repo)](#repositories-gh-repo)
  - [Create Repository](#create-repository)
  - [Clone Repository](#clone-repository)
  - [List Repositories](#list-repositories)
  - [View Repository](#view-repository)
  - [Edit Repository](#edit-repository)
  - [Delete Repository](#delete-repository)
  - [Fork Repository](#fork-repository)
  - [Sync Fork](#sync-fork)
  - [Set Default Repository](#set-default-repository)
  - [Repository Autolinks](#repository-autolinks)
  - [Repository Deploy Keys](#repository-deploy-keys)
  - [Gitignore and License](#gitignore-and-license)
- [Issues (gh issue)](#issues-gh-issue)
  - [Create Issue](#create-issue)
  - [List Issues](#list-issues)
  - [View Issue](#view-issue)
  - [Edit Issue](#edit-issue)
  - [Close/Reopen Issue](#closereopen-issue)
  - [Comment on Issue](#comment-on-issue)
  - [Issue Status](#issue-status)
  - [Pin/Unpin Issues](#pinunpin-issues)
  - [Lock/Unlock Issue](#lockunlock-issue)
  - [Transfer Issue](#transfer-issue)
  - [Delete Issue](#delete-issue)
  - [Develop Issue (Draft PR)](#develop-issue-draft-pr)

## Browse (gh browse)

```bash
# Open repository in browser
gh browse

# Open specific path
gh browse script/
gh browse main.go:312

# Open issue or PR
gh browse 123

# Open commit
gh browse 77507cd94ccafcf568f8560cfecde965fcfa63

# Open with specific branch
gh browse main.go --branch bug-fix

# Open different repository
gh browse --repo owner/repo

# Open specific pages
gh browse --actions       # Actions tab
gh browse --projects      # Projects tab
gh browse --releases      # Releases tab
gh browse --settings      # Settings page
gh browse --wiki          # Wiki page

# Print URL instead of opening
gh browse --no-browser
```

## Repositories (gh repo)

### Create Repository

```bash
# Create new repository
gh repo create my-repo

# Create with description
gh repo create my-repo --description "My awesome project"

# Create public repository
gh repo create my-repo --public

# Create private repository
gh repo create my-repo --private

# Create with homepage
gh repo create my-repo --homepage https://example.com

# Create with license
gh repo create my-repo --license mit

# Create with gitignore
gh repo create my-repo --gitignore python

# Initialize as template repository
gh repo create my-repo --template

# Create repository in organization
gh repo create org/my-repo

# Create without cloning locally
gh repo create my-repo --source=.

# Disable issues
gh repo create my-repo --disable-issues

# Disable wiki
gh repo create my-repo --disable-wiki
```

### Clone Repository

```bash
# Clone repository
gh repo clone owner/repo

# Clone to specific directory
gh repo clone owner/repo my-directory

# Clone with different branch
gh repo clone owner/repo --branch develop
```

### List Repositories

```bash
# List all repositories
gh repo list

# List repositories for owner
gh repo list owner

# Limit results
gh repo list --limit 50

# Public repositories only
gh repo list --public

# Source repositories only (not forks)
gh repo list --source

# JSON output
gh repo list --json name,visibility,owner

# Table output
gh repo list --limit 100 | tail -n +2

# Filter with jq
gh repo list --json name --jq '.[].name'
```

### View Repository

```bash
# View repository details
gh repo view

# View specific repository
gh repo view owner/repo

# JSON output
gh repo view --json name,description,defaultBranchRef

# View in browser
gh repo view --web
```

### Edit Repository

```bash
# Edit description
gh repo edit --description "New description"

# Set homepage
gh repo edit --homepage https://example.com

# Change visibility
gh repo edit --visibility private
gh repo edit --visibility public

# Enable/disable features
gh repo edit --enable-issues
gh repo edit --disable-issues
gh repo edit --enable-wiki
gh repo edit --disable-wiki
gh repo edit --enable-projects
gh repo edit --disable-projects

# Set default branch
gh repo edit --default-branch main

# Rename repository
gh repo rename new-name

# Archive repository
gh repo archive
gh repo unarchive
```

### Delete Repository

```bash
# Delete repository
gh repo delete owner/repo

# Confirm without prompt
gh repo delete owner/repo --yes
```

### Fork Repository

```bash
# Fork repository
gh repo fork owner/repo

# Fork to organization
gh repo fork owner/repo --org org-name

# Clone after forking
gh repo fork owner/repo --clone

# Remote name for fork
gh repo fork owner/repo --remote-name upstream
```

### Sync Fork

```bash
# Sync fork with upstream
gh repo sync

# Sync specific branch
gh repo sync --branch feature

# Force sync
gh repo sync --force
```

### Set Default Repository

```bash
# Set default repository for current directory
gh repo set-default

# Set default explicitly
gh repo set-default owner/repo

# Unset default
gh repo set-default --unset
```

### Repository Autolinks

```bash
# List autolinks
gh repo autolink list

# Add autolink
gh repo autolink add \
  --key-prefix JIRA- \
  --url-template https://jira.example.com/browse/<num>

# Delete autolink
gh repo autolink delete 12345
```

### Repository Deploy Keys

```bash
# List deploy keys
gh repo deploy-key list

# Add deploy key
gh repo deploy-key add ~/.ssh/id_rsa.pub \
  --title "Production server" \
  --read-only

# Delete deploy key
gh repo deploy-key delete 12345
```

### Gitignore and License

```bash
# View gitignore template
gh repo gitignore

# View license template
gh repo license mit

# License with full name
gh repo license mit --fullname "John Doe"
```

## Issues (gh issue)

### Create Issue

```bash
# Create issue interactively
gh issue create

# Create with title
gh issue create --title "Bug: Login not working"

# Create with title and body
gh issue create \
  --title "Bug: Login not working" \
  --body "Steps to reproduce..."

# Create with body from file
gh issue create --body-file issue.md

# Create with labels
gh issue create --title "Fix bug" --labels bug,high-priority

# Create with assignees
gh issue create --title "Fix bug" --assignee user1,user2

# Create in specific repository
gh issue create --repo owner/repo --title "Issue title"

# Create issue from web
gh issue create --web
```

### List Issues

```bash
# List all open issues
gh issue list

# List all issues (including closed)
gh issue list --state all

# List closed issues
gh issue list --state closed

# Limit results
gh issue list --limit 50

# Filter by assignee
gh issue list --assignee username
gh issue list --assignee @me

# Filter by labels
gh issue list --labels bug,enhancement

# Filter by milestone
gh issue list --milestone "v1.0"

# Search/filter
gh issue list --search "is:open is:issue label:bug"

# JSON output
gh issue list --json number,title,state,author

# Table view
gh issue list --json number,title,labels --jq '.[] | [.number, .title, .labels[].name] | @tsv'

# Show comments count
gh issue list --json number,title,comments --jq '.[] | [.number, .title, .comments]'

# Sort by
gh issue list --sort created --order desc
```

### View Issue

```bash
# View issue
gh issue view 123

# View with comments
gh issue view 123 --comments

# View in browser
gh issue view 123 --web

# JSON output
gh issue view 123 --json title,body,state,labels,comments

# View specific fields
gh issue view 123 --json title --jq '.title'
```

### Edit Issue

```bash
# Edit interactively
gh issue edit 123

# Edit title
gh issue edit 123 --title "New title"

# Edit body
gh issue edit 123 --body "New description"

# Add labels
gh issue edit 123 --add-label bug,high-priority

# Remove labels
gh issue edit 123 --remove-label stale

# Add assignees
gh issue edit 123 --add-assignee user1,user2

# Remove assignees
gh issue edit 123 --remove-assignee user1

# Set milestone
gh issue edit 123 --milestone "v1.0"
```

### Close/Reopen Issue

```bash
# Close issue
gh issue close 123

# Close with comment
gh issue close 123 --comment "Fixed in PR #456"

# Reopen issue
gh issue reopen 123
```

### Comment on Issue

```bash
# Add comment
gh issue comment 123 --body "This looks good!"

# Edit comment
gh issue comment 123 --edit 456789 --body "Updated comment"

# Delete comment
gh issue comment 123 --delete 456789
```

### Issue Status

```bash
# Show issue status summary
gh issue status

# Status for specific repository
gh issue status --repo owner/repo
```

### Pin/Unpin Issues

```bash
# Pin issue (pinned to repo dashboard)
gh issue pin 123

# Unpin issue
gh issue unpin 123
```

### Lock/Unlock Issue

```bash
# Lock conversation
gh issue lock 123

# Lock with reason
gh issue lock 123 --reason off-topic

# Unlock
gh issue unlock 123
```

### Transfer Issue

```bash
# Transfer to another repository
gh issue transfer 123 --repo owner/new-repo
```

### Delete Issue

```bash
# Delete issue
gh issue delete 123

# Confirm without prompt
gh issue delete 123 --yes
```

### Develop Issue (Draft PR)

```bash
# Create draft PR from issue
gh issue develop 123

# Create in specific branch
gh issue develop 123 --branch fix/issue-123

# Create with base branch
gh issue develop 123 --base main
```

