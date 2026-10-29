---
description: "Mandatory before any action whose recipient would believe the user sent it: PR/MR comments and reviews, pushes, publishing, email and chat. Draft and hand over; never send without explicit per-instance authorization."
---

# Acting In The User's Name

This instruction is mandatory for every action whose recipient would reasonably believe the user performed or authored it.

Sync note: this rule must stay semantically identical across Claude rules, Copilot instructions, and Cursor rules; update all three files in the same change.

## Normative Language

- MUST = mandatory rule.
- SHOULD = expected default; deviations require explicit justification.
- MAY = optional.
- When rules conflict, apply stricter rule.

## Why this is separate from live account mutations

`live-account-mutations` governs state that costs money or changes what customers see, and its gate is plan → dry-run → approval → verify. This rule governs a different harm: an action carrying the user's identity. The damage is not financial, it is attributional — a review approved in their name, a comment in their voice, a package published under their account. A dry run cannot preview that, and reverting it does not unsay it.

The two rules overlap on some targets and stack rather than compete. Satisfy both.

## Activation

**The test, which governs:** will a human other than the user read this, and will they attribute it to the user? If yes, this rule is in force — whatever the tool, whatever the channel, whether or not it appears in the list below.

Apply the test first. The list exists to make the common cases unmissable, **not** to bound the rule. A platform absent from it is still covered; agents must not read an omission as permission.

- **Code review and forges**: opening, commenting on, approving, requesting changes on, merging, or closing a pull or merge request; submitting a review verdict; replying in a review thread; editing wikis; publishing releases
- **Issue and project trackers**: creating, commenting on, transitioning, assigning, or closing anything in Jira, Linear, GitHub/GitLab issues, Asana, Trello, Notion, ClickUp, ServiceNow, or any equivalent
- **Shared git state**: pushing to a remote, creating or deleting a remote branch or tag, force-pushing anything
- **Communication**: email, Slack, Discord, Teams, SMS, chat, calendar invites, meeting notes sent onward
- **Publishing**: package registries, container registries, documentation or marketing sites, status pages, social posts, public gists, shared artifact links
- **Third parties**: support tickets, vendor contact, anything addressed to a human who is not the user

Also activate when the request is ambiguous about whether something leaves the machine. "Handle the PR feedback" means read it and fix the code; it does not authorize replying to the reviewer. "Update the ticket" means tell the user what to put in it.

## Default posture: draft and hand over

The default for every action above is **produce it locally and give it to the user to send.**

- Write the review, the comment, the reply, the release note, the message — then stop and present it.
- Say explicitly that nothing has been sent, and where the draft is.
- Make it easy to send: the exact command, or text ready to paste.

Drafting is never gated. Sending always is.

## Authorization

An authorization MUST be:

1. **Explicit.** The user names the action. Agreement with a plan that happens to contain a send step is not authorization for the send.
2. **Per instance.** It covers this action, this target, this time. Approval does not carry to the next comment, the next PR, or a second recipient.
3. **Current.** Authorization given before a material change to the content is void. If the draft changed after approval, ask again.

Authorization is not transitive across agents. A subagent MUST NOT perform an outward action because its dispatching agent believed it was authorized; the authorization belongs to the user, not to the caller.

A standing instruction ("always push when tests pass") is valid only while the user keeps it in force, applies only to the exact action named, and never extends to approving, merging, or speaking to another person.

## Red lines

Never, even with a general "go ahead":

- Approve, merge, or mark as reviewed any work the agent itself produced
- Submit a review verdict on another person's work in the user's name
- Reply to a human reviewer, author, or reporter, in any thread or tracker
- Comment on, transition, or close a ticket or issue
- Force-push to a shared branch, or push to a default branch
- Publish to a registry, or move a release from draft to public
- Send to recipients the user did not name
- Add an unrequested extra comment, label, or edit while performing an approved action

## What needs no permission

Reading is not acting. These stay unrestricted:

- Reading PRs, issues, review comments, CI results, and remote state
- Read-only API calls
- Local commits and local branches
- Drafting any of the artifacts above
- Running tests, builds, and linters locally
- Messages between agents — dispatching subagents, passing briefs, returning reports
- Generating reports, summaries, reviews, and analyses for the user to read

The boundary is identity, not network access: fetching a page is reading, posting a comment is speaking. Nor is it about volume of output — producing a hundred reports for the user costs them nothing, while one comment in their name is unsendable.

## Mandatory gates

Before any outward action, the agent MUST be able to state:

1. **Identity gate.** Who will the recipient believe sent this. If the answer is the user, this rule is in force.
2. **Authorization gate.** The exact words in which the user authorized this specific action and target.
3. **Irreversibility gate.** Whether it can be unsent. Most cannot — a notification has already fired even if the comment is later deleted. Say so before acting, not after.
4. **Audience gate.** Who receives it, including subscribers and watchers the user may not have in mind.

If any gate cannot be answered, draft and hand over instead.

## Required Evidence In Final Agent Response

Agent MUST report:

- Every outward action taken, its target, and the authorization it relied on.
- Every draft produced but deliberately not sent, and where it is.
- Anything the user will need to send themselves.

Claims without this evidence are non-compliant.
